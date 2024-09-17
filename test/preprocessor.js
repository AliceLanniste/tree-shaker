// @ts-check

const { treeShake } = require('@kermanx/tree-shaker')
const pc = require("picocolors");
const Diff = require('diff')
const process = require('process');
const path = require('path');

const do_minify = false;

function treeShakeEval(input, tree_shake) {
  return input.replace(/eval\('(.*)'\)/, (_, content) => {treeShake(content, tree_shake, do_minify, true)});
}

function printDiff(diff) {
  let t1 = ""
  diff.forEach((part) => {
    // green for additions, red for deletions
    t1 += part.added ? "" :
               part.removed ? pc.bgRed(part.value) :
                              part.value;
  });
  console.log("OLD", t1);
  
  let t2 = ""
  diff.forEach((part) => {
    // green for additions, red for deletions
    t2 += part.added ? pc.bgGreen(part.value) :
               part.removed ? "" :
                              part.value;
  });
  console.log("NEW", t2);
}

const total = 51617;
let executed = 0;
let skipped = 0;
let unimplemented = 0;
let minifiedTotal = 0;
let treeShakedTotal = 0;
module.exports = function(test) {
  try {
    let prelude = test.contents.slice(0, test.insertionIndex);
    let main = test.contents.slice(test.insertionIndex);

    if (
         main.includes('eval(')
      || main.includes('new Function(')
      || main.includes('$DONOTEVALUATE')
      || /with\s*\(/.test(main)
      || main.includes('noStrict')
    ) {
      skipped++;
      if (!process.stdout.isTTY) {
        console.log(`\n[SKIP] ${test.file}\n`)
      }
      return test;
    }

    if (
      main.includes('.call(')
    ) {
      skipped++;
      unimplemented++;
      if (!process.stdout.isTTY) {
        console.log(`\n[SKIP] ${test.file}\n`)
      }
      return test;
    }

    executed++;

    let progress = ((executed + skipped) * 100 / total).toFixed(2) + '%';
    let rate = (treeShakedTotal * 100 / minifiedTotal).toFixed(2) + '%';
    
    if (process.stdout.isTTY) {
      process.stdout.clearLine(0);
      process.stdout.cursorTo(0);
      process.stdout.write(`${pc.green(executed)}/${pc.white(total)} ${pc.yellow(progress)} ${pc.blue(rate)}`.padEnd(70, ' ')+path.basename(test.file));
    }

    let minified = treeShake(treeShakeEval(main, false), false, do_minify, false);
    let startTime = Date.now();
    let treeShaked = treeShake(treeShakeEval(main, true), true, do_minify, false);
    let endTime = Date.now();

    minifiedTotal += minified.length;
    treeShakedTotal += treeShaked.length;

    // console.log(`${pc.gray(main.length)} -> ${pc.red(minified.length)} -> ${pc.green(treeShaked.length)} (${pc.yellow((treeShaked.length * 100 / minified.length).toFixed(2) + '%')}) +${endTime - startTime}ms`);
    // if (minified !== treeShaked && !test.file.includes('unicode'))
    //   printDiff(Diff.diffChars(minified.slice(0, 500), treeShaked.slice(0, 500)));
    test.contents = prelude + treeShaked;
  } catch (error) {
    test.result = {
      stderr: `${error.name}: ${error.message}\n`,
      stdout: '',
      error
    };
  }

  return test;
};

process.addListener('beforeExit', () => {
  let rate = (treeShakedTotal * 100 / minifiedTotal).toFixed(2) + '%';
  process.stdout.write(`\nTreeshake rate: ${rate}\n`);
  process.stdout.write(`Transformed: ${executed}, Skipped: ${skipped}, Unimplemented: ${unimplemented}\n`);
})
