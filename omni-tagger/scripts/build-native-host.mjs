import { execSync } from 'node:child_process';
import { copyFileSync, existsSync, writeFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(__dirname, '..');
const srcTauriPath = resolve(projectRoot, 'src-tauri');

console.log('Detecting target triple...');
let targetTriple = '';
try {
  const output = execSync('rustc -vV', { encoding: 'utf-8' });
  const match = output.match(/^host: (.+)$/m);
  if (match) {
    targetTriple = match[1];
  } else {
    throw new Error('Could not parse target triple from rustc output');
  }
} catch (e) {
  console.error('Failed to get target triple:', e);
  process.exit(1);
}

console.log(`Target triple: ${targetTriple}`);

// Use src-tauri root instead of bin/ subdirectory to avoid WiX issues
const binDir = srcTauriPath;

const isWindows = process.platform === 'win32';
const ext = isWindows ? '.exe' : '';
const destPath = resolve(binDir, `native_host-${targetTriple}${ext}`);

// Create a dummy file if it doesn't exist, to satisfy tauri-build during cargo build
if (!existsSync(destPath)) {
  console.log(`Creating dummy file at ${destPath} to satisfy tauri-build...`);
  writeFileSync(destPath, isWindows ? '' : '#!/bin/sh\necho "dummy"');
  if (!isWindows) {
      try {
        execSync(`chmod +x ${destPath}`);
      } catch (e) {
        console.error('Failed to chmod dummy file:', e);
      }
  }
}

console.log('Building native_host...');
try {
  execSync('cargo build --bin native_host --release', {
    cwd: srcTauriPath,
    stdio: 'inherit',
  });
} catch (e) {
  console.error('Failed to build native_host:', e);
  process.exit(1);
}

const srcPath = resolve(srcTauriPath, 'target', 'release', `native_host${ext}`);

if (!existsSync(srcPath)) {
  console.error(`Source binary not found at: ${srcPath}`);
  process.exit(1);
}

console.log(`Copying ${srcPath} to ${destPath}`);
copyFileSync(srcPath, destPath);
if (!isWindows) {
    try {
      execSync(`chmod +x ${destPath}`);
    } catch (e) {
      console.error('Failed to chmod binary:', e);
    }
}
console.log('Done.');
