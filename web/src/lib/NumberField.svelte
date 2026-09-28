<script lang="ts">
  /** A number with its unit inside the box: "₪ 10,000", "10 %". A text
   * field rather than a number one, to show thousands separators. Commas
   * typed or left in are ignored, and the text is tidied on leaving; not
   * while editing, which would lose the selection and move the caret. The
   * value is null while the field is empty or isn't a number. `label` is its
   * name for screen readers, when no <label> names it. */
  let {
    id,
    value = $bindable(),
    prefix = '',
    suffix = '',
    step = 1,
    label,
    placeholder,
  }: {
    id: string
    value: number | null
    prefix?: string
    suffix?: string
    step?: number
    label?: string
    placeholder?: string
  } = $props()

  const format = (number: number | null) =>
    number === null ? '' : number.toLocaleString('en-US', { maximumFractionDigits: 4 })

  function parse(text: string): number | null {
    const cleaned = text.replaceAll(',', '').trim()
    if (cleaned === '') return null
    const number = Number(cleaned)
    return Number.isFinite(number) ? number : null
  }

  let focused = $state(false)
  let text = $state('')
  // Follows the value when it changes from outside, e.g. downloaded rates.
  $effect(() => {
    if (!focused) text = format(value)
  })

  function onKeyDown(event: KeyboardEvent) {
    const direction = { ArrowUp: 1, ArrowDown: -1 }[event.key]
    if (!direction) return
    event.preventDefault()
    // Rounded, so 0.1 steps don't drift to 0.30000000000000004.
    value = Math.round(((value ?? 0) + direction * step) * 1e6) / 1e6
    text = format(value)
  }
</script>

<div class="field">
  {#if prefix}<span class="unit" aria-hidden="true">{prefix}</span>{/if}
  <input
    {id}
    type="text"
    inputmode="decimal"
    autocomplete="off"
    aria-label={label}
    {placeholder}
    bind:value={text}
    oninput={() => (value = parse(text))}
    onfocus={() => (focused = true)}
    onblur={() => {
      focused = false
      text = format(value) || text
    }}
    onkeydown={onKeyDown}
  />
  {#if suffix}<span class="unit" aria-hidden="true">{suffix}</span>{/if}
</div>

<style>
  /* Looks like one input: the border is on the box around it. */
  .field {
    display: flex;
    align-items: center;
    gap: 6px;
    box-sizing: border-box;
    width: 100%;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--raised);
    transition:
      border-color var(--quick),
      box-shadow var(--quick);
  }
  .field:hover {
    border-color: var(--strong-border);
  }
  .field:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgb(123 155 255 / 0.25);
  }
  input {
    flex: 1;
    min-width: 0;
    padding: 6px 0;
    border: none;
    outline: none;
    background: transparent;
    font-variant-numeric: tabular-nums;
  }
  input::placeholder {
    color: var(--weak);
    opacity: 0.7;
  }
  .unit {
    color: var(--weak);
    user-select: none;
  }
</style>
