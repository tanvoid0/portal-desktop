/**
 * Pipeline Template Service
 * Generates pipeline templates based on frameworks and project types
 */

import { logger } from "$lib/domains/shared";
import type {
  Pipeline,
  PipelineStep,
  PipelineVariable,
  ExecutionContext,
  PipelineTemplate,
} from "../types";
import { PipelineStepType } from "../types";

const log = logger.createScoped("PipelineTemplateService");

// Re-export the PipelineTemplate from types to ensure consistency
export type { PipelineTemplate } from "../types";

class PipelineTemplateService {
  private static instance: PipelineTemplateService;
  private templates: Map<string, PipelineTemplate[]> = new Map();

  static getInstance(): PipelineTemplateService {
    if (!PipelineTemplateService.instance) {
      PipelineTemplateService.instance = new PipelineTemplateService();
      PipelineTemplateService.instance.initializeTemplates();
    }
    return PipelineTemplateService.instance;
  }

  private initializeTemplates() {
    // React/Next.js Templates
    this.addTemplate({
      key: "react-build",
      name: "React Build Pipeline",
      description: "Build and test a React application",
      framework: "react",
      category: "build",
      steps: [
        {
          key: "install-deps",
          name: "Install Dependencies",
          type: PipelineStepType.COMMAND,
          config: { command: "npm install" },
          enabled: true,
        },
        {
          key: "lint-code",
          name: "Lint Code",
          type: PipelineStepType.COMMAND,
          config: { command: "npm run lint" },
          dependsOn: ["install-deps"],
          enabled: true,
        },
        {
          key: "run-tests",
          name: "Run Tests",
          type: PipelineStepType.COMMAND,
          config: { command: "npm test" },
          dependsOn: ["install-deps"],
          enabled: true,
        },
        {
          key: "build-project",
          name: "Build Project",
          type: PipelineStepType.COMMAND,
          config: { command: "npm run build" },
          dependsOn: ["lint-code", "run-tests"],
          enabled: true,
        },
      ],
      variables: [
        {
          name: "NODE_VERSION",
          type: "string",
          defaultValue: "18",
          description: "Node.js version",
        },
        {
          name: "BUILD_ENV",
          type: "string",
          defaultValue: "production",
          description: "Build environment",
        },
      ],
      executionContext: {
        type: "sdk",
        sdkType: "node",
        workingDirectory: "${PROJECT_PATH}",
      },
      tags: ["react", "build", "test"],
    });

    // Next.js Templates
    this.addTemplate({
      key: "nextjs-full",
      name: "Next.js Full Pipeline",
      description:
        "Complete Next.js CI/CD pipeline with build, test, and deploy",
      framework: "nextjs",
      category: "ci-cd",
      steps: [
        {
          key: "install-deps",
          name: "Install Dependencies",
          type: PipelineStepType.COMMAND,
          config: { command: "npm ci" },
          enabled: true,
        },
        {
          key: "type-check",
          name: "Type Check",
          type: PipelineStepType.COMMAND,
          config: { command: "npm run type-check" },
          dependsOn: ["install-deps"],
          enabled: true,
        },
        {
          key: "lint",
          name: "Lint",
          type: PipelineStepType.COMMAND,
          config: { command: "npm run lint" },
          dependsOn: ["install-deps"],
          enabled: true,
        },
        {
          key: "run-tests",
          name: "Run Tests",
          type: PipelineStepType.COMMAND,
          config: { command: "npm test" },
          dependsOn: ["install-deps"],
          enabled: true,
        },
        {
          key: "build",
          name: "Build",
          type: PipelineStepType.COMMAND,
          config: { command: "npm run build" },
          dependsOn: ["type-check", "lint", "run-tests"],
          enabled: true,
        },
        {
          key: "build-docker-image",
          name: "Build Docker Image",
          type: PipelineStepType.DOCKER_COMMAND,
          config: {
            image: "${PROJECT_NAME}:${BUILD_NUMBER}",
            buildContext: "${PROJECT_PATH}",
            dockerfilePath: "Dockerfile",
          },
          dependsOn: ["build"],
          enabled: true,
        },
      ],
      variables: [
        {
          name: "NODE_VERSION",
          type: "string",
          defaultValue: "18",
          description: "Node.js version",
        },
        {
          name: "BUILD_NUMBER",
          type: "string",
          defaultValue: "${GIT_COMMIT_SHORT}",
          description: "Build number",
        },
      ],
      executionContext: {
        type: "sdk",
        sdkType: "node",
        workingDirectory: "${PROJECT_PATH}",
      },
      tags: ["nextjs", "ci-cd", "docker"],
    });

    // Vue.js Templates
    this.addTemplate({
      key: "vue-build",
      name: "Vue.js Build Pipeline",
      description: "Build and test a Vue.js application",
      framework: "vue",
      category: "build",
      steps: [
        {
          key: "install-deps",
          name: "Install Dependencies",
          type: PipelineStepType.COMMAND,
          config: { command: "npm install" },
          enabled: true,
        },
        {
          key: "lint",
          name: "Lint",
          type: PipelineStepType.COMMAND,
          config: { command: "npm run lint" },
          dependsOn: ["install-deps"],
          enabled: true,
        },
        {
          key: "unit-tests",
          name: "Unit Tests",
          type: PipelineStepType.COMMAND,
          config: { command: "npm run test:unit" },
          dependsOn: ["install-deps"],
          enabled: true,
        },
        {
          key: "build",
          name: "Build",
          type: PipelineStepType.COMMAND,
          config: { command: "npm run build" },
          dependsOn: ["lint", "unit-tests"],
          enabled: true,
        },
      ],
      executionContext: {
        type: "sdk",
        sdkType: "node",
        workingDirectory: "${PROJECT_PATH}",
      },
      tags: ["vue", "build", "test"],
    });

    // Node.js/Express Templates
    this.addTemplate({
      key: "nodejs-api",
      name: "Node.js API Pipeline",
      description: "Build, test, and deploy a Node.js API",
      framework: "nodejs",
      category: "full-stack",
      steps: [
        {
          key: "install-deps",
          name: "Install Dependencies",
          type: PipelineStepType.COMMAND,
          config: { command: "npm ci" },
          enabled: true,
        },
        {
          key: "run-tests",
          name: "Run Tests",
          type: PipelineStepType.COMMAND,
          config: { command: "npm test" },
          dependsOn: ["install-deps"],
          enabled: true,
        },
        {
          key: "build",
          name: "Build",
          type: PipelineStepType.COMMAND,
          config: { command: "npm run build" },
          dependsOn: ["run-tests"],
          enabled: true,
        },
        {
          key: "docker-build",
          name: "Docker Build",
          type: PipelineStepType.DOCKER_COMMAND,
          config: {
            image: "${PROJECT_NAME}-api:latest",
            buildContext: "${PROJECT_PATH}",
          },
          dependsOn: ["build"],
          enabled: true,
        },
      ],
      executionContext: {
        type: "sdk",
        sdkType: "node",
        workingDirectory: "${PROJECT_PATH}",
      },
      tags: ["nodejs", "api", "docker"],
    });

    // Python/Django Templates
    this.addTemplate({
      key: "django-full",
      name: "Django Full Pipeline",
      description: "Complete Django CI/CD pipeline",
      framework: "django",
      category: "ci-cd",
      steps: [
        {
          key: "install-deps",
          name: "Install Dependencies",
          type: PipelineStepType.SDK_COMMAND,
          config: {
            sdkType: "python",
            command: "pip",
            args: ["install", "-r", "requirements.txt"],
          },
          enabled: true,
        },
        {
          key: "run-migrations",
          name: "Run Migrations",
          type: PipelineStepType.SDK_COMMAND,
          config: {
            sdkType: "python",
            command: "python",
            args: ["manage.py", "migrate"],
          },
          dependsOn: ["install-deps"],
          enabled: true,
        },
        {
          key: "run-tests",
          name: "Run Tests",
          type: PipelineStepType.SDK_COMMAND,
          config: {
            sdkType: "python",
            command: "pytest",
          },
          dependsOn: ["install-deps"],
          enabled: true,
        },
        {
          key: "collect-static",
          name: "Collect Static Files",
          type: PipelineStepType.SDK_COMMAND,
          config: {
            sdkType: "python",
            command: "python",
            args: ["manage.py", "collectstatic", "--noinput"],
          },
          dependsOn: ["run-tests"],
          enabled: true,
        },
      ],
      executionContext: {
        type: "sdk",
        sdkType: "python",
        workingDirectory: "${PROJECT_PATH}",
      },
      tags: ["django", "python", "ci-cd"],
    });

    // Rust/Cargo Templates
    this.addTemplate({
      key: "rust-build",
      name: "Rust Build Pipeline",
      description: "Build, test, and lint a Rust project",
      framework: "rust",
      category: "build",
      steps: [
        {
          key: "format-check",
          name: "Format Check",
          type: PipelineStepType.COMMAND,
          config: { command: "cargo fmt -- --check" },
          enabled: true,
        },
        {
          key: "clippy-lint",
          name: "Clippy Lint",
          type: PipelineStepType.COMMAND,
          config: { command: "cargo clippy -- -D warnings" },
          dependsOn: ["format-check"],
          enabled: true,
        },
        {
          key: "run-tests",
          name: "Run Tests",
          type: PipelineStepType.COMMAND,
          config: { command: "cargo test" },
          dependsOn: ["format-check"],
          enabled: true,
        },
        {
          key: "build-release",
          name: "Build Release",
          type: PipelineStepType.COMMAND,
          config: { command: "cargo build --release" },
          dependsOn: ["clippy-lint", "run-tests"],
          enabled: true,
        },
      ],
      executionContext: {
        type: "sdk",
        sdkType: "rust",
        workingDirectory: "${PROJECT_PATH}",
      },
      tags: ["rust", "cargo", "build"],
    });

    // Docker-only Templates
    this.addTemplate({
      key: "docker-build-deploy",
      name: "Docker Build & Deploy",
      description: "Build and deploy using Docker",
      framework: "docker",
      category: "deploy",
      steps: [
        {
          key: "build-image",
          name: "Build Image",
          type: PipelineStepType.DOCKER_COMMAND,
          config: {
            image: "${PROJECT_NAME}:${VERSION}",
            buildContext: "${PROJECT_PATH}",
            dockerfilePath: "Dockerfile",
          },
          enabled: true,
        },
        {
          key: "run-container",
          name: "Run Container",
          type: PipelineStepType.DOCKER_COMMAND,
          config: {
            image: "${PROJECT_NAME}:${VERSION}",
            ports: ["${PORT}:8080"],
          },
          dependsOn: ["build-image"],
          enabled: true,
        },
      ],
      variables: [
        {
          name: "VERSION",
          type: "string",
          defaultValue: "latest",
          description: "Image version",
        },
        {
          name: "PORT",
          type: "number",
          defaultValue: 8080,
          description: "Host port",
        },
      ],
      executionContext: {
        type: "docker",
        workingDirectory: "${PROJECT_PATH}",
      },
      tags: ["docker", "deploy"],
    });

    // Spring Boot - Maven Templates
    this.addTemplate({
      key: "spring-boot-maven-full",
      name: "Spring Boot Maven Full Pipeline",
      description:
        "Complete Spring Boot CI/CD pipeline with Maven (clean, test, build, package, docker)",
      framework: "spring-boot",
      category: "ci-cd",
      packageManager: "maven",
      steps: [
        {
          key: "maven-clean",
          name: "Maven Clean",
          type: PipelineStepType.COMMAND,
          config: { command: "mvn clean" },
          enabled: true,
        },
        {
          key: "maven-compile",
          name: "Maven Compile",
          type: PipelineStepType.COMMAND,
          config: { command: "mvn compile" },
          dependsOn: ["maven-clean"],
          enabled: true,
        },
        {
          key: "maven-test",
          name: "Maven Test",
          type: PipelineStepType.COMMAND,
          config: { command: "mvn test" },
          dependsOn: ["maven-compile"],
          enabled: true,
        },
        {
          key: "maven-package",
          name: "Maven Package",
          type: PipelineStepType.COMMAND,
          config: { command: "mvn package -DskipTests" },
          dependsOn: ["maven-test"],
          enabled: true,
        },
        {
          key: "docker-build",
          name: "Build Docker Image",
          type: PipelineStepType.DOCKER_COMMAND,
          config: {
            image: "${PROJECT_NAME}:${BUILD_NUMBER}",
            buildContext: "${PROJECT_PATH}",
            dockerfilePath: "Dockerfile",
          },
          dependsOn: ["maven-package"],
          enabled: true,
        },
        {
          key: "manual-verification",
          name: "Manual Verification",
          type: PipelineStepType.MANUAL_APPROVAL,
          config: {
            message:
              "Please verify the build artifacts and Docker image before deployment. Review test results, build logs, and image contents.",
          },
          dependsOn: ["docker-build"],
          enabled: true,
          requiresApproval: true,
          approvalMessage:
            "Please verify the build artifacts and Docker image before deployment. Review test results, build logs, and image contents.",
        },
      ],
      variables: [
        {
          name: "JAVA_VERSION",
          type: "string",
          defaultValue: "17",
          description: "Java version",
        },
        {
          name: "SPRING_PROFILE",
          type: "string",
          defaultValue: "prod",
          description: "Spring profile",
        },
        {
          name: "BUILD_NUMBER",
          type: "string",
          defaultValue: "${GIT_COMMIT_SHORT}",
          description: "Build number",
        },
      ],
      executionContext: {
        type: "sdk",
        sdkType: "java",
        sdkVersion: "${JAVA_VERSION}",
        workingDirectory: "${PROJECT_PATH}",
      },
      tags: ["spring-boot", "maven", "java", "ci-cd", "docker"],
    });

    // Spring Boot - Gradle Templates
    this.addTemplate({
      key: "spring-boot-gradle-full",
      name: "Spring Boot Gradle Full Pipeline",
      description:
        "Complete Spring Boot CI/CD pipeline with Gradle (clean, test, build, bootJar, docker)",
      framework: "spring-boot",
      category: "ci-cd",
      packageManager: "gradle",
      steps: [
        {
          key: "gradle-clean",
          name: "Gradle Clean",
          type: PipelineStepType.COMMAND,
          config: { command: "./gradlew clean" },
          enabled: true,
        },
        {
          key: "gradle-test",
          name: "Gradle Test",
          type: PipelineStepType.COMMAND,
          config: { command: "./gradlew test" },
          dependsOn: ["gradle-clean"],
          enabled: true,
        },
        {
          key: "gradle-bootJar",
          name: "Gradle BootJar",
          type: PipelineStepType.COMMAND,
          config: { command: "./gradlew bootJar" },
          dependsOn: ["gradle-test"],
          enabled: true,
        },
        {
          key: "docker-build",
          name: "Build Docker Image",
          type: PipelineStepType.DOCKER_COMMAND,
          config: {
            image: "${PROJECT_NAME}:${BUILD_NUMBER}",
            buildContext: "${PROJECT_PATH}",
            dockerfilePath: "Dockerfile",
          },
          dependsOn: ["gradle-bootJar"],
          enabled: true,
        },
        {
          key: "manual-verification",
          name: "Manual Verification",
          type: PipelineStepType.MANUAL_APPROVAL,
          config: {
            message:
              "Please verify the build artifacts and Docker image before deployment. Review test results, build logs, and image contents.",
          },
          dependsOn: ["docker-build"],
          enabled: true,
          requiresApproval: true,
          approvalMessage:
            "Please verify the build artifacts and Docker image before deployment. Review test results, build logs, and image contents.",
        },
      ],
      variables: [
        {
          name: "JAVA_VERSION",
          type: "string",
          defaultValue: "17",
          description: "Java version",
        },
        {
          name: "SPRING_PROFILE",
          type: "string",
          defaultValue: "prod",
          description: "Spring profile",
        },
        {
          name: "BUILD_NUMBER",
          type: "string",
          defaultValue: "${GIT_COMMIT_SHORT}",
          description: "Build number",
        },
      ],
      executionContext: {
        type: "sdk",
        sdkType: "java",
        sdkVersion: "${JAVA_VERSION}",
        workingDirectory: "${PROJECT_PATH}",
      },
      tags: ["spring-boot", "gradle", "java", "ci-cd", "docker"],
    });

    // Spring Boot Microservices Template
    this.addTemplate({
      key: "spring-boot-microservices",
      name: "Spring Boot Microservices Pipeline",
      description:
        "Complete microservices pipeline with service discovery, API gateway, and database migrations",
      framework: "spring-boot",
      category: "full-stack",
      steps: [
        {
          key: "maven-clean",
          name: "Maven Clean",
          type: PipelineStepType.COMMAND,
          config: { command: "mvn clean" },
          enabled: true,
        },
        {
          key: "maven-compile",
          name: "Maven Compile",
          type: PipelineStepType.COMMAND,
          config: { command: "mvn compile" },
          dependsOn: ["maven-clean"],
          enabled: true,
        },
        {
          key: "liquibase-validate",
          name: "Liquibase Validate",
          type: PipelineStepType.COMMAND,
          config: { command: "mvn liquibase:validate" },
          dependsOn: ["maven-compile"],
          enabled: true,
        },
        {
          key: "liquibase-test",
          name: "Liquibase Test",
          type: PipelineStepType.COMMAND,
          config: { command: "mvn liquibase:updateTestingRollback" },
          dependsOn: ["liquibase-validate"],
          enabled: true,
        },
        {
          key: "liquibase-update",
          name: "Liquibase Update",
          type: PipelineStepType.COMMAND,
          config: { command: "mvn liquibase:update" },
          dependsOn: ["liquibase-test"],
          enabled: true,
        },
        {
          key: "maven-test",
          name: "Maven Test",
          type: PipelineStepType.COMMAND,
          config: { command: "mvn test" },
          dependsOn: ["maven-compile"],
          enabled: true,
        },
        {
          key: "maven-package",
          name: "Maven Package",
          type: PipelineStepType.COMMAND,
          config: { command: "mvn package -DskipTests" },
          dependsOn: ["maven-test", "liquibase-update"],
          enabled: true,
        },
        {
          key: "docker-build",
          name: "Build Docker Image",
          type: PipelineStepType.DOCKER_COMMAND,
          config: {
            image: "${SERVICE_NAME}:${BUILD_NUMBER}",
            buildContext: "${PROJECT_PATH}",
            dockerfilePath: "Dockerfile",
          },
          dependsOn: ["maven-package"],
          enabled: true,
        },
        {
          key: "manual-verification",
          name: "Manual Verification",
          type: PipelineStepType.MANUAL_APPROVAL,
          config: {
            message:
              "Please verify the build artifacts and Docker image before deployment. Check logs, test results, and image contents.",
          },
          dependsOn: ["docker-build"],
          enabled: true,
          requiresApproval: true,
          approvalMessage:
            "Please verify the build artifacts and Docker image before deployment. Check logs, test results, and image contents.",
        },
        {
          key: "docker-push",
          name: "Push Docker Image",
          type: PipelineStepType.DOCKER_COMMAND,
          config: {
            image: "${SERVICE_NAME}:${BUILD_NUMBER}",
          },
          dependsOn: ["manual-verification"],
          enabled: true,
        },
      ],
      variables: [
        {
          name: "JAVA_VERSION",
          type: "string",
          defaultValue: "17",
          description: "Java version",
        },
        {
          name: "SPRING_PROFILE",
          type: "string",
          defaultValue: "prod",
          description: "Spring profile",
        },
        {
          name: "SERVICE_NAME",
          type: "string",
          defaultValue: "${PROJECT_NAME}",
          description: "Microservice name",
        },
        {
          name: "BUILD_NUMBER",
          type: "string",
          defaultValue: "${GIT_COMMIT_SHORT}",
          description: "Build number",
        },
        {
          name: "EUREKA_URL",
          type: "string",
          defaultValue: "http://eureka-server:8761/eureka",
          description: "Eureka server URL",
        },
      ],
      executionContext: {
        type: "sdk",
        sdkType: "java",
        sdkVersion: "${JAVA_VERSION}",
        workingDirectory: "${PROJECT_PATH}",
      },
      tags: [
        "spring-boot",
        "microservices",
        "maven",
        "docker",
        "eureka",
        "liquibase",
      ],
    });

    // Spring Boot API Gateway Template
    this.addTemplate({
      key: "spring-boot-api-gateway",
      name: "Spring Boot API Gateway Pipeline",
      description:
        "Pipeline for Spring Cloud Gateway or Zuul API Gateway service",
      framework: "spring-boot",
      category: "full-stack",
      steps: [
        {
          key: "maven-clean",
          name: "Maven Clean",
          type: PipelineStepType.COMMAND,
          config: { command: "mvn clean" },
          enabled: true,
        },
        {
          key: "maven-compile",
          name: "Maven Compile",
          type: PipelineStepType.COMMAND,
          config: { command: "mvn compile" },
          dependsOn: ["maven-clean"],
          enabled: true,
        },
        {
          key: "maven-test",
          name: "Maven Test",
          type: PipelineStepType.COMMAND,
          config: { command: "mvn test" },
          dependsOn: ["maven-compile"],
          enabled: true,
        },
        {
          key: "maven-package",
          name: "Maven Package",
          type: PipelineStepType.COMMAND,
          config: { command: "mvn package -DskipTests" },
          dependsOn: ["maven-test"],
          enabled: true,
        },
        {
          key: "docker-build",
          name: "Build Docker Image",
          type: PipelineStepType.DOCKER_COMMAND,
          config: {
            image: "${GATEWAY_NAME}:${BUILD_NUMBER}",
            buildContext: "${PROJECT_PATH}",
            dockerfilePath: "Dockerfile",
          },
          dependsOn: ["maven-package"],
          enabled: true,
        },
      ],
      variables: [
        {
          name: "JAVA_VERSION",
          type: "string",
          defaultValue: "17",
          description: "Java version",
        },
        {
          name: "GATEWAY_NAME",
          type: "string",
          defaultValue: "api-gateway",
          description: "API Gateway service name",
        },
        {
          name: "BUILD_NUMBER",
          type: "string",
          defaultValue: "${GIT_COMMIT_SHORT}",
          description: "Build number",
        },
      ],
      executionContext: {
        type: "sdk",
        sdkType: "java",
        sdkVersion: "${JAVA_VERSION}",
        workingDirectory: "${PROJECT_PATH}",
      },
      tags: [
        "spring-boot",
        "api-gateway",
        "spring-cloud-gateway",
        "microservices",
      ],
    });
  }

  private addTemplate(template: PipelineTemplate) {
    const framework = template.framework || "other";
    if (!this.templates.has(framework)) {
      this.templates.set(framework, []);
    }
    this.templates.get(framework)!.push(template);
  }

  /**
   * Get all templates for a specific framework
   */
  getTemplatesForFramework(framework: string): PipelineTemplate[] {
    return this.templates.get(framework) || [];
  }

  /**
   * Get all available templates
   */
  getAllTemplates(): PipelineTemplate[] {
    const all: PipelineTemplate[] = [];
    for (const templates of this.templates.values()) {
      all.push(...templates);
    }
    return all;
  }

  /**
   * Get template by key
   */
  getTemplateByKey(key: string): PipelineTemplate | undefined {
    for (const templates of this.templates.values()) {
      const template = templates.find((t) => t.key === key);
      if (template) return template;
    }
    return undefined;
  }

  /**
   * Get template by key (alias for getTemplateByKey for backward compatibility)
   */
  getTemplateById(key: string): PipelineTemplate | undefined {
    return this.getTemplateByKey(key);
  }

  /**
   * Generate a pipeline from a template
   */
  generatePipelineFromTemplate(
    templateKey: string,
    projectId: string,
    projectName: string,
    customizations?: {
      variables?: Record<string, string | number | boolean>;
      enabledSteps?: string[];
    },
  ): Omit<Pipeline, "id" | "created_at" | "updated_at"> {
    const template = this.getTemplateByKey(templateKey);
    if (!template) {
      throw new Error(`Template with key '${templateKey}' not found`);
    }

    // Generate step IDs and apply dependencies
    // First pass: create a map of step keys to generated IDs
    const stepKeyToId = new Map<string, string>();
    template.steps.forEach((step: { key: string; name: string }) => {
      // Use the step's key as the base for the ID, or generate from name if key not provided
      const stepId = step.key || step.name.toLowerCase().replace(/\s+/g, "-");
      stepKeyToId.set(step.key, stepId);
    });

    // Second pass: create steps with proper dependency IDs
    // For template-generated steps, we use a special blockId pattern based on step type
    const steps: PipelineStep[] = template.steps.map(
      (step: {
        key: string;
        name: string;
        type: PipelineStepType;
        config: Record<string, any>;
        dependsOn?: string[];
        requiresApproval?: boolean;
        approvalMessage?: string;
      }) => {
        // Use step key as ID, or generate from name
        const stepId = step.key || step.name.toLowerCase().replace(/\s+/g, "-");
        const dependsOn = step.dependsOn?.map((depKey: string) => {
          // Find the step ID for the dependency key
          return stepKeyToId.get(depKey) || depKey;
        });

        // Generate a blockId based on step type for template-generated steps
        // This allows the execution engine to handle them appropriately
        const blockId =
          step.type === PipelineStepType.MANUAL_APPROVAL
            ? "manual-approval"
            : `template-${step.type}-${stepId}`;

        return {
          id: stepId,
          blockId,
          name: step.name,
          config: step.config,
          dependsOn: dependsOn || [],
          requiresApproval: step.requiresApproval || false,
          approvalMessage: step.approvalMessage,
        };
      },
    );

    // Apply variable customizations
    const variables: PipelineVariable[] =
      template.variables?.map(
        (v: {
          name: string;
          type: "string" | "number" | "boolean";
          defaultValue?: string | number | boolean;
          description?: string;
        }) => ({
          name: v.name,
          type: v.type as "string" | "number" | "boolean",
          value:
            customizations?.variables?.[v.name]?.toString() ||
            v.defaultValue?.toString() ||
            "",
          description: v.description,
          scope: "pipeline" as const,
        }),
      ) || [];

    return {
      projectId,
      name: `${projectName} - ${template.name}`,
      description: template.description,
      steps,
      variables,
      secrets: [],
      executionContext: template.executionContext || {
        type: "sdk",
        workingDirectory: "${PROJECT_PATH}",
      },
      enabled: true,
      createdAt: new Date(),
      updatedAt: new Date(),
    };
  }

  /**
   * Get recommended templates for a project based on its framework
   */
  getRecommendedTemplates(projectFramework?: string): PipelineTemplate[] {
    if (!projectFramework) {
      return this.getAllTemplates();
    }

    const frameworkTemplates = this.getTemplatesForFramework(projectFramework);
    const allTemplates = this.getAllTemplates();

    // Return framework-specific templates first, then others
    return [
      ...frameworkTemplates,
      ...allTemplates.filter((t) => t.framework !== projectFramework),
    ];
  }

  /**
   * Export template to JSON
   */
  exportTemplate(templateKey: string): string {
    const template = this.getTemplateByKey(templateKey);
    if (!template) {
      throw new Error(`Template with key '${templateKey}' not found`);
    }
    return JSON.stringify(template, null, 2);
  }

  /**
   * Import template from JSON
   */
  importTemplate(jsonString: string): PipelineTemplate {
    try {
      const template = JSON.parse(jsonString) as PipelineTemplate;

      // Validate required fields
      if (
        !template.key ||
        !template.name ||
        !template.description ||
        !template.steps
      ) {
        throw new Error("Invalid template format: missing required fields");
      }

      // Check if template with this key already exists
      const existing = this.getTemplateByKey(template.key);
      if (existing && !existing.id) {
        // Hardcoded template exists, throw error
        throw new Error(
          `Template with key '${template.key}' already exists as a built-in template`,
        );
      }

      // Add the imported template
      this.addTemplate(template);
      return template;
    } catch (error) {
      if (error instanceof SyntaxError) {
        throw new Error("Invalid JSON format");
      }
      throw error;
    }
  }

  /**
   * Delete a template (only if it's not a built-in template)
   */
  deleteTemplate(templateKey: string): boolean {
    const template = this.getTemplateByKey(templateKey);
    if (!template) {
      return false;
    }

    // Don't allow deletion of built-in templates (those without id)
    if (!template.id) {
      throw new Error("Cannot delete built-in templates");
    }

    // Remove from the map
    const frameworkTemplates = this.templates.get(template.framework || "");
    if (frameworkTemplates) {
      const index = frameworkTemplates.findIndex((t) => t.key === templateKey);
      if (index >= 0) {
        frameworkTemplates.splice(index, 1);
        return true;
      }
    }
    return false;
  }

  /**
   * Update a template (only if it's not a built-in template)
   */
  updateTemplate(
    templateKey: string,
    updates: Partial<PipelineTemplate>,
  ): PipelineTemplate {
    const template = this.getTemplateByKey(templateKey);
    if (!template) {
      throw new Error(`Template with key '${templateKey}' not found`);
    }

    // Don't allow updating built-in templates (those without id)
    if (!template.id) {
      throw new Error(
        "Cannot update built-in templates. Create a new template instead.",
      );
    }

    // Update the template
    const updatedTemplate: PipelineTemplate = {
      ...template,
      ...updates,
      key: template.key, // Preserve the key
    };

    // Replace in the map
    const frameworkTemplates = this.templates.get(template.framework || "");
    if (frameworkTemplates) {
      const index = frameworkTemplates.findIndex((t) => t.key === templateKey);
      if (index >= 0) {
        frameworkTemplates[index] = updatedTemplate;
        return updatedTemplate;
      }
    }

    throw new Error("Failed to update template");
  }
}

export const pipelineTemplateService = PipelineTemplateService.getInstance();
