#!/usr/bin/env node

/**
 * Copies the compiled Rust binary to bin/ with platform-specific naming
 */

import { copyFileSync, existsSync, mkdirSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';
import { platform, arch } from 'os';
import { execFileSync } from 'child_process';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, '..');

// Resolve the real target dir via cargo metadata so a global target-dir
// (e.g. in ~/.cargo/config.toml) is respected
const manifestPath = join(projectRoot, 'cli/Cargo.toml');
const metadataJson = execFileSync(
  'cargo',
  ['metadata', '--format-version', '1', '--no-deps', '--manifest-path', manifestPath],
  { encoding: 'utf8' }
);
const targetDirectory = JSON.parse(metadataJson).target_directory;

const sourcePath = join(targetDirectory, 'release/agent-browser-session');
const binDir = join(projectRoot, 'bin');

// Determine platform suffix
const platformKey = `${platform()}-${arch()}`;
const ext = platform() === 'win32' ? '.exe' : '';
const targetName = `agent-browser-session-${platformKey}${ext}`;
const targetPath = join(binDir, targetName);

if (!existsSync(sourcePath)) {
  console.error(`Error: Native binary not found at ${sourcePath}`);
  console.error('Run "cargo build --release --manifest-path cli/Cargo.toml" first');
  process.exit(1);
}

if (!existsSync(binDir)) {
  mkdirSync(binDir, { recursive: true });
}

copyFileSync(sourcePath, targetPath);
console.log(`✓ Copied native binary to ${targetPath}`);
