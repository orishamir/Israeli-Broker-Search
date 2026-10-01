<script lang="ts">
  import { formatNumber, parseNumber, stepped } from './numbers'

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

  let focused = $state(false)
  let text = $state('')
  // Follows the value when it changes from outside, e.g. downloaded rates.
  $effect(() => {
    if (!focused) text = formatNumber(value)
  })

  function onKeyDown(event: KeyboardEvent) {
    const directions: Record<string, 1 | -1> = { ArrowUp: 1, ArrowDown: -1 }
    const direction = directions[event.key]
    if (!direction) return
    event.preventDefault()
    value = stepped(value, step, direction)
    text = formatNumber(value)
  }
</script>

<!-- The number beside its unit: at the start after ₪ or $, at the end
     before %, "% בשנה" or "חודשים". -->
<div class="field" class:after={suffix && !prefix}>
  {#if prefix}<span class="unit" aria-hidden="true">{prefix}</span>{/if}
  <input
    {id}
    type="text"
    inputmode="decimal"
    autocomplete="off"
    aria-label={label}
    {placeholder}
    bind:value={text}
    oninput={() => (value = parseNumber(text))}
    onfocus={() => (focused = true)}
    onblur={() => {
      focused = false
      text = formatNumber(value) || text
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
    box-shadow: var(--focus-ring);
  }
  /* Bare: the box around it is the field (the page's text fields are
     styled in app.css, and this undoes it, focus ring included). */
  input {
    flex: 1;
    min-width: 0;
    padding: 6px 0;
    border: none;
    outline: none;
    background: transparent;
    box-shadow: none;
    font-variant-numeric: tabular-nums;
  }
  .after input {
    text-align: end;
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
