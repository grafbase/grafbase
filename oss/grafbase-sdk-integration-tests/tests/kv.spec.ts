import { expect, test } from 'vitest'

import { MF_JS_WORKER, MF_RUST_WORKER, MF_RUST_WORKER_MISSING_BASE_PREFIX } from './mf'

test('rust_worker: kv value not found', async () => {
  const key = 'rust_1'

  const resp = await MF_RUST_WORKER.dispatchFetch(`https://fake.host/kv/${key}`)
  const response = await resp.text()

  expect(resp.status).toBe(200)
  expect(response).toBe('not found')
})

test('rust_worker: missing base prefix', async () => {
  const key = 'key'
  const resp = await MF_RUST_WORKER_MISSING_BASE_PREFIX.dispatchFetch(`https://fake.host/kv/${key}`)
  const response = await resp.text()

  expect(resp.status).toBe(500)
  expect(response).toBe('KV_BASE_PREFIX env var should not be empty')
})

test('rust_worker: put and get', async () => {
  const key = 'rust_2'
  const url = `https://fake.host/kv/${key}`

  // get: not found
  let resp = await MF_RUST_WORKER.dispatchFetch(url)
  let response = await resp.text()

  expect(resp.status).toBe(200)
  expect(response).toBe('not found')

  // put
  resp = await MF_RUST_WORKER.dispatchFetch(url, { method: 'POST', body: 'kv' })
  response = await resp.text()

  expect(resp.status).toBe(200)
  expect(response).toBe('')

  // get
  resp = await MF_RUST_WORKER.dispatchFetch(url)
  response = await resp.text()

  expect(resp.status).toBe(200)
  expect(response).toBe('kv')
})

test('rust_worker: put and get metadata', async () => {
  const key = 'rust_3'
  const url = `https://fake.host/kv/${key}/metadata`

  // get: not found
  let resp = await MF_RUST_WORKER.dispatchFetch(url)
  let response = await resp.text()

  expect(resp.status).toBe(200)
  expect(response).toBe('not found')

  // put
  const body = { value: 'kv', metadata: { test: true } }

  resp = await MF_RUST_WORKER.dispatchFetch(url, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
    },
    body: JSON.stringify(body),
  })
  response = await resp.text()

  expect(resp.status).toBe(200)
  expect(response).toBe('')

  // get
  resp = await MF_RUST_WORKER.dispatchFetch(url)
  response = await resp.text()

  expect(resp.status).toBe(200)
  expect(response).toBe('{"value":"kv","metadata":"{\\"test\\":true}"}')
})

test('rust_worker: put and delete', async () => {
  const key = 'rust_4'
  const url = `https://fake.host/kv/${key}`

  // put
  let resp = await MF_RUST_WORKER.dispatchFetch(url, {
    method: 'POST',
    body: 'hello',
  })
  let response = await resp.text()

  expect(resp.status).toBe(200)
  expect(response).toBe('')

  // get
  resp = await MF_RUST_WORKER.dispatchFetch(url)
  response = await resp.text()

  expect(resp.status).toBe(200)
  expect(response).toBe('hello')

  // delete
  resp = await MF_RUST_WORKER.dispatchFetch(url, {
    method: 'DELETE',
  })
  response = await resp.text()

  expect(resp.status).toBe(200)
  expect(response).toBe('')

  // get: not found
  resp = await MF_RUST_WORKER.dispatchFetch(url)
  response = await resp.text()

  expect(resp.status).toBe(200)
  expect(response).toBe('not found')
})

test('rust_worker: list', async () => {
  // put #1
  const key_1 = 'rust_5'
  const url_1 = `https://fake.host/kv/${key_1}`
  let resp = await MF_RUST_WORKER.dispatchFetch(url_1, {
    method: 'POST',
    body: 'hello',
  })
  let response = await resp.text()

  expect(resp.status).toBe(200)
  expect(response).toBe('')

  // put #2 (w/ metadata)
  const key_2 = 'random_6'
  const url_2 = `https://fake.host/kv/${key_2}/metadata`
  const body = { value: 'list', metadata: { test: true } }

  resp = await MF_RUST_WORKER.dispatchFetch(url_2, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
    },
    body: JSON.stringify(body),
  })
  response = await resp.text()

  expect(resp.status).toBe(200)
  expect(response).toBe('')

  // list all
  resp = await MF_RUST_WORKER.dispatchFetch('https://fake.host/kv')
  response = await resp.text()
  const json = JSON.parse(response)

  expect(resp.status).toBe(200)
  expect(json.keys.length >= 2).toBe(true)

  // list prefix
  resp = await MF_RUST_WORKER.dispatchFetch('https://fake.host/kv?prefix=random')
  response = await resp.text()

  expect(resp.status).toBe(200)
  expect(response).toBe(
    '{"cursor":null,"list_complete":true,"keys":[{"name":"test/random_6","expiration":null,"metadata":"{\\"test\\":true}"}]}'
  )
})

test('js_worker: kv value not found', async () => {
  const resp = await MF_JS_WORKER.dispatchFetch('https://fake.host/kv/js_1')
  const response = await resp.text()

  expect(resp.status).toBe(200)
  expect(response).toBe('not found')
})
