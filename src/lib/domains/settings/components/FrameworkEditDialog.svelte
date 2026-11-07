<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import * as Dialog from '$lib/components/ui/dialog';
  import Search from '~icons/fa6-solid/magnifying-glass';
  import Upload from '~icons/fa6-solid/upload';
  import X from '~icons/fa6-solid/xmark';
  import Icon from '@iconify/svelte';
  import { open as openFilePicker } from '@tauri-apps/plugin-dialog';
  import { toast } from '$lib/utils/toast';
  import { ideService, type Framework } from '$lib/domains/ide';

  interface Props {
    open?: boolean;
    /** Framework being edited; null when creating. */
    framework: Framework | null;
    onSaved?: () => void;
  }

  let { open = $bindable(false), framework, onSaved = () => {} }: Props = $props();

  let name = $state('');
  let icon = $state('');
  let iconType = $state<'devicon' | 'file'>('devicon');
  let category = $state('Custom');
  let iconSearchQuery = $state('');
  let iconSearchResults = $state<string[]>([]);

  // Runs before render so the fields are filled by the time the dialog content mounts.
  $effect.pre(() => {
    if (open) {
      name = framework?.name ?? '';
      icon = framework?.icon ?? '';
      iconType = framework?.icon_type ?? 'devicon';
      category = framework?.category ?? 'Custom';
      iconSearchQuery = '';
      iconSearchResults = [];
    }
  });

  const commonIcons = [
    'logos:react',
    'logos:vue',
    'logos:angular-icon',
    'logos:svelte-icon',
    'logos:nextjs-icon',
    'logos:nodejs-icon',
    'logos:python',
    'logos:java',
    'logos:go',
    'logos:rust',
    'logos:php',
    'logos:ruby',
    'logos:django-icon',
    'logos:flask',
    'logos:laravel',
    'logos:rails',
    'logos:spring-icon',
    'logos:express',
    'logos:fastify-icon',
    'logos:nestjs',
    'logos:nuxt-icon',
    'logos:remix-icon',
    'logos:gatsby',
    'logos:quarkus-icon',
    'logos:phoenix',
    'devicon-elixir-plain colored',
    'logos:sinatra',
    'logos:symfony',
    'logos:fastapi-icon',
  ];

  function searchIcons() {
    const query = iconSearchQuery.trim() ? iconSearchQuery.toLowerCase() : null;
    iconSearchResults = query
      ? commonIcons
          .filter((i) => i.includes(query) || i.replace('logos:', '').includes(query))
          .slice(0, 12)
      : [];
  }

  function pickIconType(type: 'devicon' | 'file') {
    iconType = type;
    iconSearchQuery = '';
    iconSearchResults = [];
  }

  async function selectIconFile() {
    try {
      const file = await openFilePicker({
        multiple: false,
        filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'svg', 'ico', 'webp'] }],
      });

      if (file && typeof file === 'string') {
        icon = file;
        iconType = 'file';
      }
    } catch (error) {
      toast.error('Failed to select icon file', error);
    }
  }

  async function save() {
    if (!name.trim()) {
      toast.error('Framework name is required');
      return;
    }

    if (!icon.trim()) {
      toast.error('Framework icon is required');
      return;
    }

    try {
      if (framework) {
        await ideService.updateFramework(framework.id, name, icon, iconType, category);
        toast.success('Framework updated successfully');
      } else {
        await ideService.createFramework(name, icon, iconType, category);
        toast.success('Framework created successfully');
      }
      onSaved();
      open = false;
    } catch (error: any) {
      toast.error('Failed to save framework', error);
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-h-[90vh] max-w-2xl overflow-y-auto">
    <Dialog.Header>
      <Dialog.Title>{framework ? 'Edit Framework' : 'Create Framework'}</Dialog.Title>
    </Dialog.Header>

    <div class="space-y-4">
      <div class="space-y-2">
        <Label for="framework-name">Framework Name</Label>
        <Input id="framework-name" bind:value={name} placeholder="e.g., MyFramework" required />
      </div>

      <div class="space-y-2">
        <Label for="framework-category">Category</Label>
        <Input
          id="framework-category"
          bind:value={category}
          placeholder="e.g., Custom, Frontend, Backend"
        />
      </div>

      <div class="space-y-2">
        <Label>Icon</Label>
        <div class="mb-2 flex gap-2">
          <Button
            type="button"
            variant={iconType === 'devicon' ? 'default' : 'outline'}
            size="sm"
            onclick={() => pickIconType('devicon')}
          >
            DevIcon
          </Button>
          <Button
            type="button"
            variant={iconType === 'file' ? 'default' : 'outline'}
            size="sm"
            onclick={() => pickIconType('file')}
          >
            <Upload class="mr-2 h-4 w-4" />
            File
          </Button>
        </div>

        {#if iconType === 'devicon'}
          <div class="space-y-2">
            <div class="flex gap-2">
              <Input
                bind:value={iconSearchQuery}
                placeholder="Search for icon (e.g., react, vue, python)"
                oninput={searchIcons}
              />
              <Button type="button" variant="outline" size="sm" onclick={searchIcons}>
                <Search class="h-4 w-4" />
              </Button>
            </div>

            {#if iconSearchResults.length > 0}
              <div class="grid max-h-48 grid-cols-4 gap-2 overflow-y-auto rounded-md border p-2">
                {#each iconSearchResults as iconName}
                  <Button
                    type="button"
                    variant={icon === iconName ? 'default' : 'outline'}
                    size="sm"
                    onclick={() => {
                      icon = iconName;
                      iconSearchQuery = '';
                      iconSearchResults = [];
                    }}
                    class="flex h-auto flex-col items-center gap-1 p-2"
                  >
                    <Icon icon={iconName} class="h-6 w-6" />
                    <span class="w-full truncate text-center text-xs"
                      >{iconName.replace('logos:', '')}</span
                    >
                  </Button>
                {/each}
              </div>
            {/if}

            {#if icon}
              <div class="flex items-center gap-3 rounded-md border p-3">
                <Icon {icon} class="h-8 w-8" />
                <div class="flex-1">
                  <p class="text-sm font-medium">{icon}</p>
                  {@render removeButton()}
                </div>
              </div>
            {:else}
              <p class="text-xs text-muted-foreground">
                Search for an icon or enter icon name (e.g., logos:react, logos:vue)
              </p>
              <Input bind:value={icon} placeholder="e.g., logos:react, logos:vue" />
            {/if}
          </div>
        {:else}
          <div class="space-y-2">
            <Button type="button" variant="outline" onclick={selectIconFile}>
              <Upload class="mr-2 h-4 w-4" />
              Select Icon File
            </Button>
            {#if icon}
              <div class="flex items-center gap-3 rounded-md border p-3">
                <img src={icon} alt="Framework icon" class="h-8 w-8" />
                <div class="flex-1">
                  <p class="truncate text-sm font-medium">{icon}</p>
                  {@render removeButton()}
                </div>
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <Dialog.Footer>
        <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
        <Button onclick={save}>{framework ? 'Update' : 'Create'} Framework</Button>
      </Dialog.Footer>
    </div>
  </Dialog.Content>
</Dialog.Root>

{#snippet removeButton()}
  <Button variant="ghost" size="sm" class="mt-1 h-6" onclick={() => (icon = '')}>
    <X class="mr-1 h-3 w-3" />
    Remove
  </Button>
{/snippet}
