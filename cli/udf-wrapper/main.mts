import { Worker } from 'worker_threads'

enum WorkerEvent {
  Exit = 'exit',
}

const [, , ...paths] = process.argv

const signals = []

for (const path of paths) {
  const worker = new Worker(path)
  const signal = new Promise((resolve) => worker.on(WorkerEvent.Exit, resolve))
  signals.push(signal)
}

await Promise.all(signals)
