import { expect, test } from 'vitest'
import type { YourPlan } from './app.svelte'
import * as core from './core/core'
import { BUYING_INTERVALS, decode, encode, type Shared } from './link'

const yours: YourPlan = {
  id: 'abc-123',
  plan: core.newPlan('My deal'),
  brokerName: 'Bank Leumi',
  basedOn: { broker: 'Bank Leumi', plan: 'Pepper' },
}

const shared: Shared = {
  security: 'IndexFund',
  exchange: 'Tlv',
  firstDeposit: 10_000,
  monthlyDeposit: 2_500.5,
  yearlyReturnPercent: 7,
  years: 25,
  buyEveryMonths: 3,
  sharePrice: 500,
  depositGrowthPercent: 3,
  inflationPercent: 2,
  sellAtEnd: false,
  plans: ['Leumi · Pepper', 'Meitav · Typical offer'],
  yours: [yours],
}

test('a link carries everything there and back, and reads as short keys', () => {
  const hash = encode(shared)
  expect(hash).toMatch(/^s=IndexFund&x=Tlv&d=10000&m=2500.5&r=7&y=25&b=3&p=500&g=3&i=2&h=1&plan=/)
  expect(decode(hash)).toEqual(shared)
  // With or without the #, as the address bar gives it.
  expect(decode(`#${hash}`)).toEqual(shared)
})

test('what is left out stays out, and selling at the end is the default that is not written', () => {
  expect(encode({})).toBe('')
  expect(decode('')).toEqual({})
  expect(encode({ sellAtEnd: true, plans: [], yours: [] })).toBe('')
  expect(decode(encode({ years: 20 }))).toEqual({ years: 20 })
})

test('nonsense is dropped, field by field', () => {
  const decoded = decode('s=Gold&x=Usa&d=-5&m=abc&y=200&b=5&p=0&g=1e999&yours=notbase64&plan=Anything')
  expect(decoded).toEqual({ exchange: 'Usa', plans: ['Anything'] })
  expect(decode('yours=' + btoa('{"not":"a list"}'))).toEqual({})
  expect(
    decode('yours=' + btoa('[1, {"id": 5}, {"id": "ok", "plan": {}, "brokerName": null, "basedOn": null}]')),
  ).toEqual({
    yours: [{ id: 'ok', plan: {}, brokerName: null, basedOn: null }],
  })
  expect(BUYING_INTERVALS.map(({ months }) => months)).toEqual([1, 2, 3, 6, 12])
})
