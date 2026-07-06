import {
  toastActions,
  type Toast,
} from "$lib/domains/shared/stores/toastStore";

export type ToastOptions = Partial<
  Pick<Toast, "duration" | "action" | "description">
>;

function resolveDescription(
  descriptionOrOptions?: string | unknown | ToastOptions,
  options?: ToastOptions,
): { description?: string; options?: ToastOptions } {
  if (
    descriptionOrOptions &&
    typeof descriptionOrOptions === "object" &&
    !Array.isArray(descriptionOrOptions) &&
    ("description" in descriptionOrOptions ||
      "duration" in descriptionOrOptions ||
      "action" in descriptionOrOptions)
  ) {
    const opts = descriptionOrOptions as ToastOptions;
    return {
      description: opts.description,
      options: opts,
    };
  }

  if (typeof descriptionOrOptions === "string") {
    return {
      description: descriptionOrOptions,
      options,
    };
  }

  return { description: undefined, options };
}

/** Unified toast API — sonner-compatible ergonomics backed by toastStore. */
export const toast = {
  show(title: string, options?: ToastOptions) {
    return toastActions.show({ title, ...options });
  },

  success(
    title: string,
    descriptionOrOptions?: string | ToastOptions,
    options?: ToastOptions,
  ) {
    const { description, options: resolvedOptions } = resolveDescription(
      descriptionOrOptions,
      options,
    );
    return toastActions.success(title, description, resolvedOptions);
  },

  error(
    title: string,
    descriptionOrOptions?: string | unknown | ToastOptions,
    options?: ToastOptions,
  ) {
    const { description, options: resolvedOptions } = resolveDescription(
      descriptionOrOptions,
      options,
    );

    if (description !== undefined) {
      return toastActions.error(title, description, resolvedOptions);
    }

    if (
      descriptionOrOptions !== undefined &&
      typeof descriptionOrOptions !== "object"
    ) {
      return toastActions.error(title, descriptionOrOptions, resolvedOptions);
    }

    return toastActions.error(title, undefined, resolvedOptions);
  },

  warning(
    title: string,
    descriptionOrOptions?: string | ToastOptions,
    options?: ToastOptions,
  ) {
    const { description, options: resolvedOptions } = resolveDescription(
      descriptionOrOptions,
      options,
    );
    return toastActions.warning(title, description, resolvedOptions);
  },

  info(
    title: string,
    descriptionOrOptions?: string | ToastOptions,
    options?: ToastOptions,
  ) {
    const { description, options: resolvedOptions } = resolveDescription(
      descriptionOrOptions,
      options,
    );
    return toastActions.info(title, description, resolvedOptions);
  },

  dismiss(id: string) {
    toastActions.dismiss(id);
  },

  clear() {
    toastActions.clear();
  },
};

export { toastActions, toastStore } from "$lib/domains/shared/stores/toastStore";
export type { Toast } from "$lib/domains/shared/stores/toastStore";
