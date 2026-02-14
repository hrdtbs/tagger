import { execSync } from 'node:child_process';
import { copyFileSync, existsSync, writeFileSync, mkdirSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(__dirname, '..');
const srcTauriPath = resolve(projectRoot, 'src-tauri');

// Create resources directory
const resourcesDir = resolve(srcTauriPath, 'resources');
if (!existsSync(resourcesDir)) {
  mkdirSync(resourcesDir, { recursive: true });
}

const destExt = '.exe';
const destPath = resolve(resourcesDir, `native_host${destExt}`);

// Create a dummy file if it doesn't exist, to satisfy tauri-build during cargo build
if (!existsSync(destPath)) {
  console.log(`Creating dummy file at ${destPath} to satisfy tauri-build...`);
  writeFileSync(destPath, '');
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

const isWindows = process.platform === 'win32';
const ext = isWindows ? '.exe' : '';
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
