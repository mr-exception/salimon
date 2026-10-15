// Pinned, local Khronos validator. No external resources or timestamped reports.
const fs = require('node:fs');
const validator = require('gltf-validator');
const expected = require('./package.json').dependencies['gltf-validator'];
if (require('gltf-validator/package.json').version !== expected) {
  throw new Error('validator version mismatch; run npm ci in models/tools/gltf');
}
validator.validateBytes(new Uint8Array(fs.readFileSync(process.argv[2])), {
  maxIssues: 0,
  externalResourceFunction: () => Promise.reject(new Error('external resources forbidden')),
}).then(report => {
  delete report.validatedAt;
  console.log(JSON.stringify(report));
}).catch(error => { console.error(error); process.exitCode = 1; });
