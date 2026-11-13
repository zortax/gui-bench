// The switch. HTML has no switch element, so it is a button with the `switch` role and a knob.

/** A switch that shows `on`; a click asks for the other state. `label` is its accessible name. */
export function Switch(props: { on: boolean; onChange: (on: boolean) => void; label: string }) {
  return (
    <button
      class="switch"
      classList={{ on: props.on }}
      role="switch"
      aria-checked={props.on}
      aria-label={props.label}
      onClick={() => props.onChange(!props.on)}
    >
      <span class="switch__knob" />
    </button>
  );
}
