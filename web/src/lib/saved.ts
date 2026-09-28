// What the app keeps in this browser between visits. Storage can be missing
// or refuse (private windows, blocked site data), so failing only loses the
// convenience.

export function load<T>(key: string, fallback: T): T {
  try {
    const saved = localStorage.getItem(key)
    return saved === null ? fallback : (JSON.parse(saved) as T)
  } catch {
    return fallback
  }
}

export function save(key: string, value: unknown) {
  try {
    localStorage.setItem(key, JSON.stringify(value))
  } catch {
    // Not saved, but the app still works.
  }
}
