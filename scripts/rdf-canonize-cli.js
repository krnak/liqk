#!/usr/bin/env node

const fs = require('fs');
const canonize = require('rdf-canonize');

async function main() {
  const args = process.argv.slice(2);

  if (args.length === 0 || args.includes('--help') || args.includes('-h')) {
    console.log(`Usage: rdf-canonize-cli <file.nt|file.nq>

Canonize N-Triples or N-Quads files using URDNA2015 algorithm.

Options:
  -h, --help    Show this help message
  -o, --output  Output file (default: stdout)`);
    process.exit(args.length === 0 ? 1 : 0);
  }

  let inputFile = null;
  let outputFile = null;

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '-o' || args[i] === '--output') {
      outputFile = args[++i];
    } else if (!args[i].startsWith('-')) {
      inputFile = args[i];
    }
  }

  if (!inputFile) {
    console.error('Error: No input file specified');
    process.exit(1);
  }

  if (!fs.existsSync(inputFile)) {
    console.error(`Error: File not found: ${inputFile}`);
    process.exit(1);
  }

  const ext = inputFile.toLowerCase().split('.').pop();
  if (ext !== 'nt' && ext !== 'nq') {
    console.error('Error: File must be .nt (N-Triples) or .nq (N-Quads)');
    process.exit(1);
  }

  try {
    const input = fs.readFileSync(inputFile, 'utf8');

    const result = await canonize.canonize(input, {
      algorithm: 'URDNA2015',
      inputFormat: 'application/n-quads'
    });

    if (outputFile) {
      fs.writeFileSync(outputFile, result);
      console.error(`Canonized output written to: ${outputFile}`);
    } else {
      process.stdout.write(result);
    }
  } catch (err) {
    console.error(`Error: ${err.message}`);
    process.exit(1);
  }
}

main();
