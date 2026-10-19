import { expect, test } from 'vitest'
import type { YourPlan } from './app.svelte'
import * as core from './core/core'
import { kindByKey, kindKey, wayKey } from './kinds'
import { BUYING_INTERVALS, decode, decodeLocation, encode, type Shared } from './link'

const yours: YourPlan = {
  id: 'abc-123',
  plan: core.newPlan('My deal'),
  brokerName: 'Bank Leumi',
  basedOn: { broker: 'Bank Leumi', plan: 'Pepper' },
}

const way = core.ways()[0].way

const shared: Shared = {
  situation: core.situations().find(({ id }) => id === 'existing')?.id ?? core.situations()[0].id,
  bank: 'MizrahiTefahot',
  now: 10_000,
  monthly: 2_500.5,
  horizon: { Months: 60 },
  flexibility: 'CanWait',
  cushion: { kind: 'hasNot', monthlyExpenses: 7_000 },
  needsAnyDay: true,
  expert: true,
  thing: 'sp500',
  ways: [wayKey(way)],
  security: 'IndexFund',
  exchange: 'Tlv',
  yearlyReturnPercent: 7,
  inflationPercent: 3,
  boiRatePercent: 4.25,
  depositGrowthPercent: 3,
  buyEveryMonths: BUYING_INTERVALS[2].months,
  sharePrice: 500,
  todaysMoney: false,
  sellAtEnd: false,
  asPension: true,
  age: 41,
  overrideArena: 'Safe',
  overrideKind: kindKey({ GovernmentBond: { linked: true } }),
  plans: ['Leumi · Pepper', 'Meitav · Typical offer'],
  yours: [yours],
}

test('a link carries every field there and back', () => {
  const hash = encode(shared)
  expect(hash.startsWith('v=2&')).toBe(true)
  expect(decode(hash)).toEqual(shared)
  // With or without the #, as the address bar gives it.
  expect(decode(`#${hash}`)).toEqual(shared)
})

test.each([
  ['months', { Months: 18 }],
  ['not known', 'Unknown'],
  ['retirement', { Retirement: { age: 41, retire_at: 67 } }],
] as const)('a horizon of %s round-trips', (_, horizon) => {
  expect(decode(encode({ horizon })).horizon).toEqual(horizon)
})

test.each([
  [{ kind: 'has' }],
  [{ kind: 'hasNot', monthlyExpenses: null }],
  [{ kind: 'hasNot', monthlyExpenses: 8_000 }],
] as const)('a cushion answer round-trips', (cushion) => {
  expect(decode(encode({ cushion })).cushion).toEqual(cushion)
})

test('a flexibility answer round-trips, both ways', () => {
  expect(decode(encode({ flexibility: 'Fixed' })).flexibility).toBe('Fixed')
  expect(decode(encode({ flexibility: 'CanWait' })).flexibility).toBe('CanWait')
})

test('every kind and way the core lists has a key that reads back', () => {
  for (const { way: each } of core.ways()) {
    expect(kindByKey(kindKey({ Way: each }))).toEqual({ Way: each })
  }
  for (const kind of ['MoneyMarketFund', 'FixedDeposit', 'LinkedDeposit', 'CurrentAccount'] as const) {
    expect(kindByKey(kindKey(kind))).toBe(kind)
  }
  expect(kindByKey(kindKey({ GovernmentBond: { linked: false } }))).toEqual({
    GovernmentBond: { linked: false },
  })
})

test('a link without v=2, or with another version, is not read', () => {
  expect(decode('n=5000&m=100&h=60')).toEqual({})
  expect(decode('v=1&n=5000')).toEqual({})
  expect(decode('s=IndexFund&x=Tlv&d=10000&y=25')).toEqual({})
  expect(decode('')).toEqual({})
})

test('a link may be in the query string, the hash first', () => {
  expect(decodeLocation({ hash: '', search: '?v=2&n=5' })).toEqual({ now: 5 })
  expect(decodeLocation({ hash: '#v=2&n=7', search: '?v=2&n=5' })).toEqual({ now: 7 })
  expect(decodeLocation({ hash: '', search: '' })).toEqual({})
})

test('what makes no sense is left out and the rest still applies', () => {
  const read = decode(
    'v=2&n=-5&m=abc&h=0&fx=maybe&cu=n:x&s=Crypto&e=Mars&b=5&p=0&a=200&th=moon&way=nothing&oa=Sea&ok=Way:nope&si=nope&bk=Nobody&r=7',
  )
  expect(read).toEqual({ yearlyReturnPercent: 7 })
})

test("a plan's id is kept once, and a list that isn't one is left out", () => {
  const twice = encode({ yours: [yours, { ...yours, brokerName: 'again' }] })
  expect(decode(twice).yours).toEqual([yours])
  expect(decode('v=2&yours=!!!').yours).toBeUndefined()
})

test('the assumptions round-trip', () => {
  const shared = { yearlyReturnPercent: 7.5, inflationPercent: 3, boiRatePercent: 2.25 }
  expect(decode(encode(shared))).toEqual(shared)
})
