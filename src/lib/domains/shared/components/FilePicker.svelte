<!-- Compatibility wrapper: keep domain import path while using canonical UI atom -->
<script lang="ts">
  import FilePicker from "$lib/components/ui/file-picker.svelte";

  interface Props {
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    required?: boolean;
    label?: string;
    description?: string;
    filters?: Array<{ name: string; extensions: string[] }>;
    selectFolder?: boolean;
    onChange?: (path: string) => void;
  }

  let {
    value = "",
    placeholder = "/path/to/file",
    disabled = false,
    required = false,
    label = "File Path",
    description,
    filters = [],
    selectFolder = false,
    onChange,
  }: Props = $props();
</script>

<FilePicker
  {value}
  {placeholder}
  {disabled}
  {required}
  {label}
  {description}
  {filters}
  {selectFolder}
  {onChange}
/>
