<!--
	Simplified Select — wraps shadcn Select primitives.

	Supports four option formats: string[], { value, label }[], constant arrays, TypeScript enums.
-->
<script lang="ts">
  import { cn } from "$lib/utils";
  import * as SelectPrimitive from "$lib/components/ui/select";

  interface SelectOption {
    value: string;
    label: string;
    disabled?: boolean;
  }

  type ConstantArray = readonly {
    readonly value: string;
    readonly label: string;
    readonly disabled?: boolean;
  }[];

  type EnumType = Record<string, string | number>;

  interface Props {
    options: string[] | SelectOption[] | ConstantArray | EnumType;
    defaultValue?: string;
    value?: string;
    placeholder?: string;
    onSelect?: (value: string) => void;
    onValueChange?: (value: string) => void;
    disabled?: boolean;
    required?: boolean;
    error?: string;
    class?: string;
  }

  let {
    options = [],
    defaultValue = "",
    value = $bindable(),
    placeholder = "Select an option...",
    onSelect = () => {},
    onValueChange,
    disabled = false,
    required = false,
    error = "",
    class: className = "",
  }: Props = $props();

  let internalValue = $state("");

  $effect(() => {
    const external =
      value !== undefined && value !== null ? value : defaultValue;
    internalValue = external ?? "";
  });

  let normalizedOptions = $derived.by(() => {
    if (typeof options === "object" && !Array.isArray(options)) {
      return Object.entries(options).map(([key, val]) => ({
        value: String(val),
        label: key
          .replace(/_/g, " ")
          .replace(/\b\w/g, (l) => l.toUpperCase()),
        disabled: false,
      }));
    }

    return options.map((option) =>
      typeof option === "string"
        ? { value: option, label: option, disabled: false }
        : { disabled: false, ...option },
    );
  });

  let displayValue = $derived.by(() => {
    if (!internalValue) return placeholder;
    const option = normalizedOptions.find(
      (opt) => opt.value === internalValue,
    );
    return option ? option.label : internalValue;
  });

  function handleValueChange(newValue: string | undefined) {
    const selected = newValue ?? "";
    if (value !== undefined) {
      value = selected;
    }
    onSelect(selected);
    onValueChange?.(selected);
  }
</script>

<div class={cn("relative w-full", className)}>
  <SelectPrimitive.Root
    type="single"
    bind:value={internalValue}
    onValueChange={handleValueChange}
    {disabled}
  >
    <SelectPrimitive.Trigger
      class={cn(
        "h-10 w-full",
        error && "border-destructive aria-invalid:border-destructive",
      )}
      aria-required={required}
      aria-invalid={!!error}
    >
      <span
        data-slot="select-value"
        class={cn(
          "block truncate",
          !internalValue && "text-muted-foreground",
        )}
      >
        {displayValue}
      </span>
    </SelectPrimitive.Trigger>
    <SelectPrimitive.Content
      class="w-full min-w-[var(--bits-select-anchor-width)]"
    >
      {#each normalizedOptions as option (option.value)}
        <SelectPrimitive.Item
          value={option.value}
          label={option.label}
          disabled={option.disabled}
        />
      {/each}
    </SelectPrimitive.Content>
  </SelectPrimitive.Root>

  {#if error}
    <p class="mt-1 text-sm text-destructive" role="alert">
      {error}
    </p>
  {/if}
</div>
