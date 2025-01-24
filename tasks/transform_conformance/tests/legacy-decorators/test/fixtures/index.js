import {  } from 'fs';
import { transpileModule } from 'typescript'
import { readdir, writeFile, rename, mkdir, readFile } from 'fs/promises';
import { basename, dirname, join } from 'path';
const __dirname = new URL('.', import.meta.url).pathname + 'typescript';


// async function rename() {
//   const files = await readdir(__dirname, {recursive: true});
//   files.map((file) => join(__dirname, file) ).forEach(async file => {
//     if (file.endsWith(".ts")) {
//       const newFile = file.replace(/\.ts$/, '/input.ts');
//       await mkdir(file.replace(/\.ts$/, ""), {recursive: true});
//       await rename(file, newFile);
//     }
//   });
// }
async function main() {
  const files = await readdir(__dirname, {recursive: true});
  files.map((file) => join(__dirname, file) ).forEach(async file => {
    console.log(file)
    if (file.endsWith("input.ts")) {
      /// Generate the output file path by using `typescript` library, and we need to set target to esnext
      let output = transpileModule(
        await readFile(file, 'utf8'),
        {
          compilerOptions: {
            target: 'esnext',
            experimentalDecorators: true,
            noEmitHelpers: true
          },
        }
      );
      let outputText = output.outputText.replaceAll('__decorate(', 'babelHelpers.decorate(').replaceAll('__param(', 'babelHelpers.decorateParam(');
      console.log(outputText)
      await writeFile(file.replace(/input\.ts$/, 'output.js'), outputText);
    }
  });
}

await main();
