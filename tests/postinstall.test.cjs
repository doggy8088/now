'use strict';

const assert = require('node:assert/strict');
const { chmodSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } = require('node:fs');
const { tmpdir } = require('node:os');
const { join } = require('node:path');
const { spawnSync } = require('node:child_process');
const test = require('node:test');

const {
  artifactName,
  cargoTarget,
  platformKey,
  releaseBaseUrl,
  sha256,
  verifyChecksum,
} = require('../npm/postinstall.cjs');

test('maps supported platforms to Rust targets', () => {
  assert.equal(platformKey('darwin', 'arm64'), 'darwin-arm64');
  assert.equal(cargoTarget('darwin', 'arm64'), 'aarch64-apple-darwin');
  assert.equal(cargoTarget('darwin', 'x64'), 'x86_64-apple-darwin');
  assert.equal(cargoTarget('linux', 'x64'), 'x86_64-unknown-linux-gnu');
  assert.equal(cargoTarget('win32', 'x64'), 'x86_64-pc-windows-msvc');
});

test('rejects unsupported platforms', () => {
  assert.throws(() => cargoTarget('linux', 'arm'), /Unsupported platform/);
});

test('formats artifact names and release URLs', () => {
  assert.equal(artifactName('x86_64-unknown-linux-gnu'), 'now-x86_64-unknown-linux-gnu.tar.xz');
  assert.equal(artifactName('x86_64-pc-windows-msvc'), 'now-x86_64-pc-windows-msvc.zip');
  assert.equal(releaseBaseUrl('1.2.3'), 'https://github.com/doggy8088/now/releases/download/v1.2.3');
});

test('verifies sha256 checksums', () => {
  const dir = mkdtempSync(join(tmpdir(), 'now-'));
  const file = join(dir, 'sample.txt');
  writeFileSync(file, 'hello');
  const digest = sha256(file);
  verifyChecksum(file, `${digest}  sample.txt`);
  assert.throws(() => verifyChecksum(file, '0'.repeat(64)), /Checksum mismatch/);
});

test('wrapper invokes installed binary on Unix-like systems', { skip: process.platform === 'win32' }, () => {
  const binDir = join(__dirname, '..', 'npm', 'now-bin');
  const bin = join(binDir, 'now');
  rmSync(binDir, { recursive: true, force: true });
  mkdirSync(binDir, { recursive: true });
  writeFileSync(bin, '#!/bin/sh\nprintf "wrapper:%s\\n" "$1"\n');
  chmodSync(bin, 0o755);

  try {
    const result = spawnSync(process.execPath, [join(__dirname, '..', 'npm', 'cli.cjs'), 'ok'], {
      encoding: 'utf8',
    });
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /wrapper:ok/);
  } finally {
    rmSync(binDir, { recursive: true, force: true });
  }
});
