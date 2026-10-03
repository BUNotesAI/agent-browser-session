#!/usr/bin/env node

/**
 * Postinstall script for agent-browser-session
 *
 * Downloads the platform-specific native binary if not present.
 */

import { existsSync, mkdirSync, chmodSync, copyFileSync, createWriteStream, mkdtempSync, readFileSync, rmSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';
import { platform, arch, tmpdir } from 'os';
import { get } from 'https';
import { pipeline } from 'stream/promises';
import { execFileSync } from 'child_process';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, '..');
const binDir = join(projectRoot, 'bin');

// Platform detection (naming must match bin/agent-browser-session and copy-native.js)
const platformKey = `${platform()}-${arch()}`;
const ext = platform() === 'win32' ? '.exe' : '';
const binaryName = `agent-browser-session-${platformKey}${ext}`;
const binaryPath = join(binDir, binaryName);

// Package info
const packageJson = JSON.parse(readFileSync(join(projectRoot, 'package.json'), 'utf8'));
const version = packageJson.version;

// GitHub release URL: releases ship a per-platform tarball, not a bare binary
const GITHUB_REPO = 'BUNotesAI/agent-browser-session';
const assetName = `agent-browser-session-${platformKey}`;
const DOWNLOAD_URL = `https://github.com/${GITHUB_REPO}/releases/download/v${version}/${assetName}.tar.gz`;

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const request = (url) => {
      get(url, (response) => {
        // Handle redirects
        if ([301, 302, 303, 307, 308].includes(response.statusCode)) {
          response.resume();
          request(response.headers.location);
          return;
        }

        if (response.statusCode !== 200) {
          response.resume();
          reject(new Error(`Failed to download: HTTP ${response.statusCode}`));
          return;
        }

        pipeline(response, createWriteStream(dest)).then(resolve, reject);
      }).on('error', reject);
    };

    request(url);
  });
}

// Download and extract in a temp dir so a failed attempt never leaves a partial file in bin/
async function installFromRelease() {
  const workDir = mkdtempSync(join(tmpdir(), 'agent-browser-session-'));
  try {
    const archivePath = join(workDir, `${assetName}.tar.gz`);
    await downloadFile(DOWNLOAD_URL, archivePath);

    const member = `${assetName}/bin/agent-browser-session${ext}`;
    execFileSync('tar', ['-xzf', archivePath, '-C', workDir, member]);
    copyFileSync(join(workDir, member), binaryPath);
  } finally {
    rmSync(workDir, { recursive: true, force: true });
  }
}

async function main() {
  // Check if binary already exists
  if (existsSync(binaryPath)) {
    console.log(`✓ Native binary already exists: ${binaryName}`);
    return;
  }

  mkdirSync(binDir, { recursive: true });

  console.log(`Downloading native binary for ${platformKey}...`);
  console.log(`URL: ${DOWNLOAD_URL}`);

  try {
    await installFromRelease();

    // Make executable on Unix
    if (platform() !== 'win32') {
      chmodSync(binaryPath, 0o755);
    }

    console.log(`✓ Downloaded native binary: ${binaryName}`);
  } catch (err) {
    console.log(`⚠ Could not download native binary: ${err.message}`);
    console.log('  The CLI cannot run until the native binary is present.');
    console.log('');
    console.log('To build the native binary locally:');
    console.log('  1. Install Rust: https://rustup.rs');
    console.log('  2. Run: npm run build:native');
  }

  // Reminder about Patchright browsers
  console.log('');
  console.log('╔═══════════════════════════════════════════════════════════════════════════╗');
  console.log('║ To download browser binaries, run:                                        ║');
  console.log('║                                                                           ║');
  console.log('║     npx patchright install chromium                                       ║');
  console.log('║                                                                           ║');
  console.log('║ On Linux, include system dependencies with:                               ║');
  console.log('║                                                                           ║');
  console.log('║     npx patchright install --with-deps chromium                           ║');
  console.log('║                                                                           ║');
  console.log('╚═══════════════════════════════════════════════════════════════════════════╝');
}

main().catch(console.error);
