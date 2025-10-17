/* eslint-env node */
/* global console, process */
import { existsSync } from 'node:fs';
import { spawnSync } from 'node:child_process';

if (!existsSync('.git')) {
  console.warn('[husky] 跳过安装：未检测到 .git 目录。');
  process.exit(0);
}

const result = spawnSync('pnpm', ['exec', 'husky', 'install'], {
  stdio: 'inherit',
  shell: process.platform === 'win32',
});

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}
