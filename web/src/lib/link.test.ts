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
  inTodaysMoney: true,
  sellAtEnd: false,
  asPension: true,
  age: 41,
  plans: ['Leumi · Pepper', 'Meitav · Typical offer'],
  yours: [yours],
}

test('a link carries everything there and back, and reads as short keys', () => {
  const hash = encode(shared)
  expect(hash).toMatch(/^s=IndexFund&x=Tlv&d=10000&m=2500.5&r=7&y=25&b=3&p=500&g=3&a=41&i=2&h=1&k=1&plan=/)
  expect(decode(hash)).toEqual(shared)
  // With or without the #, as the address bar gives it.
  expect(decode(`#${hash}`)).toEqual(shared)
})

test("an inflation alone is for the tax; with today's money it keeps the key older links used", () => {
  expect(encode({ inflationPercent: 3 })).toBe('f=3')
  expect(decode('f=3')).toEqual({ inflationPercent: 3 })
  expect(encode({ inflationPercent: 3, inTodaysMoney: true })).toBe('i=3')
  // A link from before the two were told apart: its amounts were in today's money.
  expect(decode('i=3')).toEqual({ inflationPercent: 3, inTodaysMoney: true })
  expect(decode('a=500&k=2')).toEqual({})
})

test('what is left out stays out, and selling at the end is the default that is not written', () => {
  expect(encode({})).toBe('')
  expect(decode('')).toEqual({})
  expect(encode({ sellAtEnd: true, plans: [], yours: [] })).toBe('')
  expect(decode(encode({ years: 20 }))).toEqual({ years: 20 })
})

test('the fund chosen for an ETF in Tel Aviv travels too, and an unknown one is dropped', () => {
  const foreign: Shared = { security: 'Etf', exchange: 'Tlv', product: 'ForeignEtfInTelAviv' }
  expect(encode(foreign)).toBe('s=Etf&x=Tlv&fund=ForeignEtfInTelAviv')
  expect(decode(encode(foreign))).toEqual(foreign)
  expect(decode('s=Etf&x=Tlv&fund=Gold')).toEqual({ security: 'Etf', exchange: 'Tlv' })
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

test('a short-term link carries its inputs, places and your deposits, and nothing of the long term', () => {
  const short: Shared = {
    family: 'short',
    short: {
      firstDeposit: 40_000,
      monthlyDeposit: 0,
      months: 24,
      ratePercent: 3,
      inflationPercent: 2.5,
      places: ['Fixed-rate deposit · Bank Leumi', 'Money market fund · Cheapest fund'],
      yours: [{ id: 'd-1', name: 'הצעה מלאומי', ratePercent: 4.1 }],
    },
  }
  const hash = encode(short)
  expect(hash).toMatch(/^c=s&d=40000&m=0&n=24&r=3&f=2.5&place=/)
  expect(decode(hash)).toEqual(short)
})

test('a short-term link keeps only what makes sense', () => {
  expect(decode('c=s&n=0&d=-5&m=x&deposits=nonsense')).toEqual({ family: 'short', short: {} })
  expect(decode('c=s&n=61')).toEqual({ family: 'short', short: {} })
  expect(decode('c=s&n=60')).toEqual({ family: 'short', short: { months: 60 } })
})
