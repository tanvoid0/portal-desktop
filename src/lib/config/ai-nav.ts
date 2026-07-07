import type { NavSection } from "$lib/components/shell/nav-types";

export const AI_PROVIDER_SETTINGS_PATH = "/settings/ai";

export function buildAiNavSections(): NavSection[] {
  return [
    {
      title: "AI",
      items: [
        {
          title: "Chat",
          url: "/ai/chat",
          icon: "message-square",
          description: "Chat with AI",
        },
        {
          title: "Training Data",
          url: "/ai/training",
          icon: "database",
          description: "Manage training data",
        },
        {
          title: "History",
          url: "/ai/history",
          icon: "history",
          description: "Browse past AI conversations",
        },
      ],
    },
  ];
}
