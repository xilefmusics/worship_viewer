import { spawn } from 'node:child_process'
import { existsSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { loadEnvFile } from 'node:process'
import { fileURLToPath } from 'node:url'

const frontendRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')

function loadIfPresent(name) {
  const file = resolve(frontendRoot, name)
  if (!existsSync(file)) return
  loadEnvFile(file)
}

// process.env > .env.local > .env  (loadEnvFile never overwrites)
loadIfPresent('.env.local')
loadIfPresent('.env')

const [cmd, ...args] = process.argv.slice(2)
if (!cmd) {
  console.error('usage: with-env.mjs <command> [...args]')
  process.exit(1)
}

const child = spawn(cmd, args, {
  stdio: 'inherit',
  env: process.env,
  shell: process.platform === 'win32',
})

child.on('error', (err) => {
  console.error(err)
  process.exit(1)
})

child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal)
    return
  }
  process.exit(code ?? 1)
})
