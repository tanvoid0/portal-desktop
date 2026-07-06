import type { NavSection } from "$lib/components/shell/nav-types";

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
          title: "Providers",
          url: "/ai/providers",
          icon: "settings",
          description: "Configure AI providers",
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
