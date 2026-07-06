import type { NavSection } from "$lib/components/shell/nav-types";

export interface DashboardNavOverview {
  running_services_count?: number;
  project_stats?: { total_projects?: number };
  task_stats?: { total?: number };
}

export function buildMainNavSections(
  overview: DashboardNavOverview | null | undefined,
): NavSection[] {
  const projectCount = overview?.project_stats?.total_projects ?? 0;
  const taskCount = overview?.task_stats?.total ?? 0;
  const runningServices = overview?.running_services_count ?? 0;

  return [
    {
      title: "Navigation",
      items: [
        {
          title: "Overview",
          url: "/",
          icon: "home",
          description: "Portal Desktop home",
          badge: null,
        },
        {
          title: "SDK Manager",
          url: "/sdk",
          icon: "code",
          description: "SDK version management",
          badge: runningServices > 0 ? runningServices : null,
          submenu: [
            {
              title: "Node.js",
              url: "/sdk/nodejs",
              icon: "node-js",
              description: "Node.js SDK management",
            },
            {
              title: "Python",
              url: "/sdk/python",
              icon: "python",
              description: "Python SDK management",
            },
            {
              title: "Java",
              url: "/sdk/java",
              icon: "coffee",
              description: "Java SDK management",
            },
            {
              title: "Rust",
              url: "/sdk/rust",
              icon: "rust",
              description: "Rust SDK management",
            },
            {
              title: "Go",
              url: "/sdk/go",
              icon: "go",
              description: "Go SDK management",
            },
          ],
        },
      ],
    },
    {
      title: "Tools",
      items: [
        {
          title: "Terminal",
          url: "/terminal",
          icon: "terminal",
          description:
            "Interactive terminal workspace with command blocks and AI",
          badge: null,
        },
        {
          title: "Projects",
          url: "/projects",
          icon: "folder",
          description: "Project management",
          badge: projectCount > 0 ? projectCount : null,
        },
        {
          title: "Pipeline Runs",
          url: "/pipeline-runs",
          icon: "history",
          description: "Global history of pipeline executions",
          badge: null,
        },
        {
          title: "Tasks",
          url: "/tasks",
          icon: "check-square",
          description: "Task management",
          badge: taskCount > 0 ? taskCount : null,
        },
        {
          title: "Credentials",
          url: "/credentials",
          icon: "lock",
          description: "Secure credential vault",
          badge: null,
        },
        {
          title: "Cloud (Kubernetes)",
          url: "/cloud",
          icon: "cloud",
          description: "Kubernetes cluster management",
          badge: null,
        },
        {
          title: "Local Docker",
          url: "/deployments",
          icon: "container",
          description: "Local Docker container management",
          badge: null,
        },
        {
          title: "Documents",
          url: "/documents",
          icon: "file-text",
          description: "Workspace documentation",
          badge: null,
        },
        {
          title: "Automation",
          url: "/automation",
          icon: "workflow",
          description: "Pipeline blocks, scripts, and custom utilities",
          badge: null,
        },
        {
          title: "Disk Utility",
          url: "/utilities/disk",
          icon: "hard-drive",
          description: "AI-assisted disk cleanup with human review",
          badge: null,
        },
        {
          title: "AI",
          url: "/ai",
          icon: "sparkles",
          description: "AI chat and management",
          badge: null,
        },
        {
          title: "Coder",
          url: "/coder",
          icon: "robot",
          description: "AI coding agent",
          badge: null,
        },
        {
          title: "Settings",
          url: "/settings",
          icon: "settings",
          description: "Application settings",
          badge: null,
        },
      ],
    },
  ];
}
