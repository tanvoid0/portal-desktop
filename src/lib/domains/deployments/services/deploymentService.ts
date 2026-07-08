/**
 * Deployment Service - Frontend business logic for deployment management
 */

import { invoke } from "@tauri-apps/api/core";
import { logger } from "$lib/domains/shared";
import type {
  Deployment,
  DockerContainer,
  DockerStatus,
  DeploymentCreateRequest,
  DeploymentUpdateRequest,
} from "../types";
import { DeploymentStatus } from "../types";

export class DeploymentService {
  private static instance: DeploymentService;

  static getInstance(): DeploymentService {
    if (!DeploymentService.instance) {
      DeploymentService.instance = new DeploymentService();
    }
    return DeploymentService.instance;
  }

  /**
   * Create a new deployment
   */
  async createDeployment(
    request: DeploymentCreateRequest,
  ): Promise<Deployment> {
    try {
      logger.info("Creating deployment", {
        context: "DeploymentService",
        name: request.name,
        projectPath: request.projectPath,
        type: request.type,
      });

      // Convert frontend request to backend format
      const backendRequest = {
        project_id: request.metadata?.projectId || "",
        name: request.name,
        deployment_type: request.type === "docker" ? "Docker" : "Cli",
        sdk_version: request.metadata?.sdkVersion || "latest", // FUTURE: Add sdkVersion to request type
        project_type: request.projectType,
        project_path: request.projectPath,
        environment: request.environment.variables || {},
        exposed_port: request.exposedPort,
        docker_image_name: request.dockerImageName,
        dockerfile_path: request.dockerfilePath,
        command: request.command,
        working_directory: request.workingDirectory,
      };

      const backendDeployment = await invoke<any>("create_deployment_command", {
        request: backendRequest,
      });

      const deployment = this.convertBackendDeployment(backendDeployment);

      logger.info("Deployment created successfully", {
        context: "DeploymentService",
        deploymentId: deployment.id,
      });

      return deployment;
    } catch (error) {
      logger.error("Failed to create deployment", {
        context: "DeploymentService",
        error,
        data: { name: request.name, projectPath: request.projectPath },
      });
      throw error;
    }
  }

  /**
   * Convert backend deployment to frontend format
   */
  private convertBackendDeployment(backendDeployment: any): Deployment {
    return {
      ...backendDeployment,
      type: backendDeployment.deployment_type === "Docker" ? "docker" : "cli",
      createdAt: new Date(backendDeployment.created_at),
      updatedAt: new Date(backendDeployment.updated_at),
      startedAt: backendDeployment.started_at
        ? new Date(backendDeployment.started_at)
        : undefined,
      stoppedAt: backendDeployment.stopped_at
        ? new Date(backendDeployment.stopped_at)
        : undefined,
      // Map backend fields to frontend
      dockerImageName: backendDeployment.docker_image_name,
      dockerfilePath: backendDeployment.dockerfile_path,
      workingDirectory: backendDeployment.working_directory,
      processId: backendDeployment.process_id,
      exposedPort: backendDeployment.exposed_port,
    } as Deployment;
  }

  /**
   * Get all deployments
   */
  async getDeployments(): Promise<Deployment[]> {
    try {
      logger.info("Getting deployments", { context: "DeploymentService" });

      const backendDeployments = await invoke<any[]>("get_deployments_command");
      const deployments = backendDeployments.map((d) =>
        this.convertBackendDeployment(d),
      );

      logger.info("Deployments retrieved", {
        context: "DeploymentService",
        data: { count: deployments.length },
      });

      return deployments;
    } catch (error) {
      logger.error("Failed to get deployments", {
        context: "DeploymentService",
        error,
      });
      throw error;
    }
  }

  /**
   * Get deployment by ID
   */
  async getDeployment(deploymentId: string): Promise<Deployment> {
    try {
      logger.info("Getting deployment", {
        context: "DeploymentService",
        data: { deploymentId },
      });

      const backendDeployment = await invoke<any>("get_deployment_command", {
        deploymentId,
      });

      if (!backendDeployment) {
        throw new Error("Deployment not found");
      }

      const deployment = this.convertBackendDeployment(backendDeployment);

      logger.info("Deployment retrieved", {
        context: "DeploymentService",
        data: { deploymentId },
      });

      return deployment;
    } catch (error) {
      logger.error("Failed to get deployment", {
        context: "DeploymentService",
        error,
        data: { deploymentId },
      });
      throw error;
    }
  }

  /**
   * Start a deployment
   */
  async startDeployment(deploymentId: string): Promise<Deployment> {
    try {
      logger.info("Starting deployment", {
        context: "DeploymentService",
        data: { deploymentId },
      });

      const backendDeployment = await invoke<any>("start_deployment_command", {
        deploymentId,
      });

      const deployment = this.convertBackendDeployment(backendDeployment);

      logger.info("Deployment started successfully", {
        context: "DeploymentService",
        data: { deploymentId },
      });

      return deployment;
    } catch (error) {
      logger.error("Failed to start deployment", {
        context: "DeploymentService",
        error,
        data: { deploymentId },
      });
      throw error;
    }
  }

  /**
   * Stop a deployment
   */
  async stopDeployment(deploymentId: string): Promise<Deployment> {
    try {
      logger.info("Stopping deployment", {
        context: "DeploymentService",
        data: { deploymentId },
      });

      const backendDeployment = await invoke<any>("stop_deployment_command", {
        deploymentId,
      });

      const deployment = this.convertBackendDeployment(backendDeployment);

      logger.info("Deployment stopped successfully", {
        context: "DeploymentService",
        data: { deploymentId },
      });

      return deployment;
    } catch (error) {
      logger.error("Failed to stop deployment", {
        context: "DeploymentService",
        error,
        data: { deploymentId },
      });
      throw error;
    }
  }

  /**
   * Delete a deployment
   */
  async deleteDeployment(deploymentId: string): Promise<void> {
    try {
      logger.info("Deleting deployment", {
        context: "DeploymentService",
        data: { deploymentId },
      });

      await invoke("delete_deployment_command", { deploymentId });

      logger.info("Deployment deleted successfully", {
        context: "DeploymentService",
        data: { deploymentId },
      });
    } catch (error) {
      logger.error("Failed to delete deployment", {
        context: "DeploymentService",
        error,
        data: { deploymentId },
      });
      throw error;
    }
  }

  /**
   * Update a deployment
   */
  async updateDeployment(
    deploymentId: string,
    request: DeploymentUpdateRequest,
  ): Promise<Deployment> {
    try {
      logger.info("Updating deployment", {
        context: "DeploymentService",
        data: { deploymentId },
      });

      const backendDeployment = await invoke<any>("update_deployment_command", {
        deploymentId,
        request,
      });

      const deployment = this.convertBackendDeployment(backendDeployment);

      logger.info("Deployment updated successfully", {
        context: "DeploymentService",
        data: { deploymentId },
      });

      return deployment;
    } catch (error) {
      logger.error("Failed to update deployment", {
        context: "DeploymentService",
        error,
        data: { deploymentId },
      });
      throw error;
    }
  }

  /**
   * Get deployment logs
   */
  async getDeploymentLogs(
    deploymentId: string,
    tail?: number,
  ): Promise<string[]> {
    try {
      logger.info("Getting deployment logs", {
        context: "DeploymentService",
        data: { deploymentId, tail },
      });

      const logs = await invoke<string[]>("get_deployment_logs_command", {
        deploymentId,
        tail,
      });

      logger.info("Deployment logs retrieved", {
        context: "DeploymentService",
        data: { deploymentId, logCount: logs.length },
      });

      return logs;
    } catch (error) {
      logger.error("Failed to get deployment logs", {
        context: "DeploymentService",
        error,
        data: { deploymentId },
      });
      throw error;
    }
  }

  /**
   * Refresh deployment statuses
   */
  async refreshDeploymentStatuses(): Promise<Deployment[]> {
    try {
      logger.info("Refreshing deployment statuses", {
        context: "DeploymentService",
      });

      const backendDeployments = await invoke<any[]>(
        "refresh_deployment_statuses_command",
      );
      const deployments = backendDeployments.map((d) =>
        this.convertBackendDeployment(d),
      );

      logger.info("Deployment statuses refreshed", {
        context: "DeploymentService",
        data: { count: deployments.length },
      });

      return deployments;
    } catch (error) {
      logger.error("Failed to refresh deployment statuses", {
        context: "DeploymentService",
        error,
      });
      throw error;
    }
  }

  /**
   * Convert backend container to frontend format
   */
  private convertBackendContainer(backendContainer: any): DockerContainer {
    const resourceStats = backendContainer.resource_stats
      ? {
          cpuPercent: backendContainer.resource_stats.cpu_percent ?? 0,
          memoryBytes: backendContainer.resource_stats.memory_bytes ?? 0,
          memoryLimitBytes:
            backendContainer.resource_stats.memory_limit_bytes ?? 0,
          memoryPercent: backendContainer.resource_stats.memory_percent ?? 0,
          networkRxBytes: backendContainer.resource_stats.network_rx_bytes ?? 0,
          networkTxBytes: backendContainer.resource_stats.network_tx_bytes ?? 0,
          blockReadBytes: backendContainer.resource_stats.block_read_bytes ?? 0,
          blockWriteBytes:
            backendContainer.resource_stats.block_write_bytes ?? 0,
        }
      : undefined;

    return {
      ...backendContainer,
      status: backendContainer.status,
      ports: Array.isArray(backendContainer.ports)
        ? backendContainer.ports
        : backendContainer.ports
          ? [backendContainer.ports]
          : [],
      createdAt: (() => {
        if (
          !backendContainer.created_at ||
          backendContainer.created_at.trim() === ""
        ) {
          return new Date();
        }
        const dateStr = backendContainer.created_at.trim();
        const unixTimestamp = parseInt(dateStr, 10);
        if (!isNaN(unixTimestamp) && unixTimestamp > 0) {
          const date = new Date(unixTimestamp * 1000);
          if (!isNaN(date.getTime())) {
            return date;
          }
        }
        const date = new Date(dateStr);
        return isNaN(date.getTime()) ? new Date() : date;
      })(),
      volumes: backendContainer.volumes || [],
      environment: backendContainer.environment || {},
      resourceStats,
    } as DockerContainer;
  }

  /**
   * Check whether an error indicates Docker is installed but not running
   */
  isDockerOfflineError(error: unknown): boolean {
    const message =
      error instanceof Error
        ? error.message
        : typeof error === "string"
          ? error
          : "";

    return (
      message.includes("dockerDesktopLinuxEngine") ||
      message.includes("Cannot connect to the Docker daemon") ||
      message.includes("failed to connect to the docker API") ||
      message.includes("The system cannot find the file specified") ||
      message.includes("Is the docker daemon running")
    );
  }

  /**
   * Get Docker daemon status
   */
  async getDockerStatus(): Promise<DockerStatus> {
    try {
      const status = await invoke<DockerStatus>("get_docker_status_command");
      logger.info("Docker status retrieved", {
        context: "DeploymentService",
        data: { running: status.running, installed: status.installed },
      });
      return status;
    } catch (error) {
      logger.error("Failed to get Docker status", {
        context: "DeploymentService",
        error,
      });
      throw error;
    }
  }

  /**
   * Start Docker Desktop / Docker daemon
   */
  async startDocker(): Promise<string> {
    try {
      const message = await invoke<string>("start_docker_command");
      logger.info("Docker start requested", {
        context: "DeploymentService",
        data: { message },
      });
      return message;
    } catch (error) {
      logger.error("Failed to start Docker", {
        context: "DeploymentService",
        error,
      });
      throw error;
    }
  }

  /**
   * List Docker containers
   */
  async listContainers(): Promise<DockerContainer[]> {
    try {
      logger.info("Listing containers", { context: "DeploymentService" });

      const backendContainers = await invoke<any[]>("list_containers_command");
      const containers = backendContainers.map((c) =>
        this.convertBackendContainer(c),
      );

      logger.info("Containers retrieved", {
        context: "DeploymentService",
        data: { count: containers.length },
      });

      return containers;
    } catch (error) {
      if (this.isDockerOfflineError(error)) {
        logger.warn("Docker is not running; skipping container list", {
          context: "DeploymentService",
        });
      } else {
        logger.error("Failed to list containers", {
          context: "DeploymentService",
          error,
        });
      }
      throw error;
    }
  }

  /**
   * Get deployment status color
   */
  getStatusColor(status: DeploymentStatus): string {
    const colors: Record<DeploymentStatus, string> = {
      [DeploymentStatus.CREATING]: "text-yellow-600",
      [DeploymentStatus.RUNNING]: "text-green-600",
      [DeploymentStatus.STOPPED]: "text-gray-600",
      [DeploymentStatus.FAILED]: "text-red-600",
      [DeploymentStatus.RESTARTING]: "text-blue-600",
      [DeploymentStatus.REMOVING]: "text-gray-400",
      [DeploymentStatus.BUILDING]: "text-yellow-600",
    };
    return colors[status] || "text-gray-400";
  }

  /**
   * Get deployment status icon
   */
  getStatusIcon(status: DeploymentStatus): string {
    const icons: Record<DeploymentStatus, string> = {
      [DeploymentStatus.CREATING]: "🔨",
      [DeploymentStatus.RUNNING]: "🟢",
      [DeploymentStatus.STOPPED]: "⏹️",
      [DeploymentStatus.FAILED]: "❌",
      [DeploymentStatus.RESTARTING]: "🔄",
      [DeploymentStatus.REMOVING]: "🗑️",
      [DeploymentStatus.BUILDING]: "🔨",
    };
    return icons[status] || "❓";
  }

  /**
   * Format deployment duration
   */
  formatDuration(createdAt: string, updatedAt: string): string {
    const created = new Date(createdAt);
    const updated = new Date(updatedAt);
    const duration = updated.getTime() - created.getTime();

    const seconds = Math.floor(duration / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d ${hours % 24}h`;
    if (hours > 0) return `${hours}h ${minutes % 60}m`;
    if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
    return `${seconds}s`;
  }

  /**
   * Validate deployment request
   */
  validateDeploymentRequest(
    request: Partial<DeploymentCreateRequest>,
  ): string[] {
    const errors: string[] = [];

    if (!request.name?.trim()) {
      errors.push("Name is required");
    }

    if (!request.projectPath?.trim()) {
      errors.push("Project path is required");
    }

    if (!request.projectType) {
      errors.push("Project type is required");
    }

    return errors;
  }

  /**
   * Generate deployment name from project
   */
  generateDeploymentName(projectName: string, projectType: string): string {
    const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, "-");
    return `${projectName}-${projectType}-${timestamp}`;
  }

  /**
   * Get project type from path
   */
  detectProjectType(projectPath: string): string {
    // This would be more sophisticated in a real implementation
    // For now, we'll do basic detection based on common files
    if (projectPath.includes("package.json")) return "node";
    if (projectPath.includes("Cargo.toml")) return "rust";
    if (
      projectPath.includes("requirements.txt") ||
      projectPath.includes("pyproject.toml")
    )
      return "python";
    if (projectPath.includes("go.mod")) return "go";
    return "unknown";
  }

  /**
   * Build Docker image with progress tracking
   */
  async buildDockerImage(
    contextPath: string,
    imageName: string,
    dockerfilePath?: string,
  ): Promise<string> {
    try {
      logger.info("Building Docker image", {
        context: "DeploymentService",
        contextPath,
        imageName,
        dockerfilePath,
      });

      const result = await invoke<string>("build_docker_image_command", {
        context_path: contextPath,
        image_name: imageName,
        dockerfile_path: dockerfilePath,
      });

      logger.info("Docker image built successfully", {
        context: "DeploymentService",
        imageName,
        result,
      });

      return result;
    } catch (error) {
      logger.error("Failed to build Docker image", {
        context: "DeploymentService",
        error,
        data: { contextPath, imageName, dockerfilePath },
      });
      throw error;
    }
  }

  /**
   * Check if CLI process is running
   */
  async getProcessStatus(deploymentId: string): Promise<boolean> {
    try {
      logger.info("Checking process status", {
        context: "DeploymentService",
        deploymentId,
      });

      const isRunning = await invoke<boolean>("get_process_status_command", {
        deployment_id: deploymentId,
      });

      return isRunning;
    } catch (error) {
      logger.error("Failed to check process status", {
        context: "DeploymentService",
        error,
        data: { deploymentId },
      });
      throw error;
    }
  }

  /**
   * Start a Docker container
   */
  async startContainer(containerId: string): Promise<void> {
    try {
      logger.info("Starting container", {
        context: "DeploymentService",
        containerId,
      });

      await invoke("start_container_command", {
        containerId: containerId,
      });

      logger.info("Container started successfully", {
        context: "DeploymentService",
        containerId,
      });
    } catch (error) {
      logger.error("Failed to start container", {
        context: "DeploymentService",
        error,
        data: { containerId },
      });
      throw error;
    }
  }

  /**
   * Stop a Docker container
   */
  async stopContainer(containerId: string): Promise<void> {
    try {
      logger.info("Stopping container", {
        context: "DeploymentService",
        containerId,
      });

      await invoke("stop_container_command", {
        containerId: containerId,
      });

      logger.info("Container stopped successfully", {
        context: "DeploymentService",
        containerId,
      });
    } catch (error) {
      logger.error("Failed to stop container", {
        context: "DeploymentService",
        error,
        data: { containerId },
      });
      throw error;
    }
  }

  /**
   * Remove a Docker container
   */
  async removeContainer(containerId: string): Promise<void> {
    try {
      logger.info("Removing container", {
        context: "DeploymentService",
        containerId,
      });

      await invoke("remove_container_command", {
        containerId: containerId,
      });

      logger.info("Container removed successfully", {
        context: "DeploymentService",
        containerId,
      });
    } catch (error) {
      logger.error("Failed to remove container", {
        context: "DeploymentService",
        error,
        data: { containerId },
      });
      throw error;
    }
  }
}

export const deploymentService = DeploymentService.getInstance();
