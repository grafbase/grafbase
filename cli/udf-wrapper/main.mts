import { Worker } from 'worker_threads'
const [, , ...paths] = process.argv

let signals = []
for (const path of paths) {
  const worker = new Worker(path)
  const signal = new Promise((resolve) => worker.on('exit', resolve))
  signals.push(signal)
}

await Promise.all(signals)
