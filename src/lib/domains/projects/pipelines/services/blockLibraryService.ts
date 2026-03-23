/**
 * Block Library Service - Management of reusable command blocks
 */

import { invoke } from '@tauri-apps/api/core';
import { logger } from '@/lib/domains/shared';
import type { Block, CreateBlockRequest } from '../types';

const log = logger.createScoped('BlockLibraryService');

export class BlockLibraryService {
	private static instance: BlockLibraryService;
	private defaultBlocks: Block[] = [];

	static getInstance(): BlockLibraryService {
		if (!BlockLibraryService.instance) {
			BlockLibraryService.instance = new BlockLibraryService();
			BlockLibraryService.instance.initializeDefaultBlocks();
		}
		return BlockLibraryService.instance;
	}

	/**
	 * Initialize default block library
	 */
	private initializeDefaultBlocks(): void {
		this.defaultBlocks = [
			// Install Dependencies
			{
				id: 'install-npm',
				name: 'Install NPM Dependencies',
				description: 'Install Node.js dependencies using npm',
				category: 'build',
				version: '1.0.0',
				parameters: [
					{
						name: 'packageManager',
						type: 'select',
						description: 'Package manager to use',
						required: true,
						defaultValue: 'npm',
						options: ['npm', 'yarn', 'pnpm'],
					},
					{
						name: 'installCommand',
						type: 'select',
						description: 'Install command variant',
						required: false,
						defaultValue: 'install',
						options: ['install', 'ci'],
					},
				],
				command: '${packageManager} ${installCommand}',
				executionType: 'command',
				defaultConfig: { packageManager: 'npm', installCommand: 'install' },
				tags: ['node', 'npm', 'dependencies', 'install'],
			},
			{
				id: 'install-pip',
				name: 'Install Python Dependencies',
				description: 'Install Python dependencies using pip',
				category: 'build',
				version: '1.0.0',
				parameters: [
					{
						name: 'requirementsFile',
						type: 'string',
						description: 'Requirements file path',
						required: false,
						defaultValue: 'requirements.txt',
					},
					{
						name: 'upgrade',
						type: 'boolean',
						description: 'Upgrade existing packages',
						required: false,
						defaultValue: false,
					},
				],
				command: 'pip install -r ${requirementsFile}${upgrade ? " --upgrade" : ""}',
				executionType: 'command',
				defaultConfig: { requirementsFile: 'requirements.txt', upgrade: false },
				tags: ['python', 'pip', 'dependencies', 'install'],
			},
			{
				id: 'install-cargo',
				name: 'Install Rust Dependencies',
				description: 'Install Rust dependencies using Cargo',
				category: 'build',
				version: '1.0.0',
				parameters: [
					{
						name: 'buildDeps',
						type: 'boolean',
						description: 'Build dependencies',
						required: false,
						defaultValue: true,
					},
				],
				command: 'cargo build${buildDeps ? " --deps" : ""}',
				executionType: 'command',
				defaultConfig: { buildDeps: true },
				tags: ['rust', 'cargo', 'dependencies', 'install'],
			},
			// Run Tests
			{
				id: 'test-npm',
				name: 'Run NPM Tests',
				description: 'Execute test suite using npm',
				category: 'test',
				version: '1.0.0',
				parameters: [
					{
						name: 'testCommand',
						type: 'string',
						description: 'Test command',
						required: false,
						defaultValue: 'test',
					},
					{
						name: 'coverage',
						type: 'boolean',
						description: 'Generate coverage report',
						required: false,
						defaultValue: false,
					},
				],
				command: 'npm run ${testCommand}${coverage ? " -- --coverage" : ""}',
				executionType: 'command',
				defaultConfig: { testCommand: 'test', coverage: false },
				tags: ['node', 'npm', 'test'],
			},
			{
				id: 'test-pytest',
				name: 'Run Pytest',
				description: 'Execute Python tests using pytest',
				category: 'test',
				version: '1.0.0',
				parameters: [
					{
						name: 'testPath',
						type: 'string',
						description: 'Test path or file',
						required: false,
						defaultValue: '.',
					},
					{
						name: 'verbose',
						type: 'boolean',
						description: 'Verbose output',
						required: false,
						defaultValue: false,
					},
				],
				command: 'pytest ${testPath}${verbose ? " -v" : ""}',
				executionType: 'command',
				defaultConfig: { testPath: '.', verbose: false },
				tags: ['python', 'pytest', 'test'],
			},
			{
				id: 'test-cargo',
				name: 'Run Cargo Tests',
				description: 'Execute Rust tests using Cargo',
				category: 'test',
				version: '1.0.0',
				parameters: [
					{
						name: 'release',
						type: 'boolean',
						description: 'Run in release mode',
						required: false,
						defaultValue: false,
					},
				],
				command: 'cargo test${release ? " --release" : ""}',
				executionType: 'command',
				defaultConfig: { release: false },
				tags: ['rust', 'cargo', 'test'],
			},
			// Lint
			{
				id: 'lint-eslint',
				name: 'Lint with ESLint',
				description: 'Run ESLint to check code quality',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'fix',
						type: 'boolean',
						description: 'Auto-fix issues',
						required: false,
						defaultValue: false,
					},
				],
				command: 'eslint .${fix ? " --fix" : ""}',
				executionType: 'command',
				defaultConfig: { fix: false },
				tags: ['node', 'eslint', 'lint'],
			},
			{
				id: 'lint-pylint',
				name: 'Lint with Pylint',
				description: 'Run Pylint to check Python code quality',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'target',
						type: 'string',
						description: 'Target file or directory',
						required: false,
						defaultValue: '.',
					},
				],
				command: 'pylint ${target}',
				executionType: 'command',
				defaultConfig: { target: '.' },
				tags: ['python', 'pylint', 'lint'],
			},
			{
				id: 'lint-clippy',
				name: 'Lint with Clippy',
				description: 'Run Clippy to check Rust code quality',
				category: 'utility',
				version: '1.0.0',
				parameters: [],
				command: 'cargo clippy -- -D warnings',
				executionType: 'command',
				defaultConfig: {},
				tags: ['rust', 'clippy', 'lint'],
			},
			// Build
			{
				id: 'build-npm',
				name: 'Build NPM Project',
				description: 'Build Node.js project',
				category: 'build',
				version: '1.0.0',
				parameters: [
					{
						name: 'buildCommand',
						type: 'string',
						description: 'Build command',
						required: false,
						defaultValue: 'build',
					},
					{
						name: 'environment',
						type: 'select',
						description: 'Build environment',
						required: false,
						defaultValue: 'production',
						options: ['development', 'staging', 'production'],
					},
				],
				command: 'npm run ${buildCommand}',
				executionType: 'command',
				defaultConfig: { buildCommand: 'build', environment: 'production' },
				tags: ['node', 'npm', 'build'],
			},
			{
				id: 'build-cargo',
				name: 'Build Cargo Project',
				description: 'Build Rust project using Cargo',
				category: 'build',
				version: '1.0.0',
				parameters: [
					{
						name: 'release',
						type: 'boolean',
						description: 'Build in release mode',
						required: false,
						defaultValue: false,
					},
				],
				command: 'cargo build${release ? " --release" : ""}',
				executionType: 'command',
				defaultConfig: { release: false },
				tags: ['rust', 'cargo', 'build'],
			},
			// Docker
			{
				id: 'docker-build',
				name: 'Build Docker Image',
				description: 'Build a Docker image',
				category: 'deploy',
				version: '1.0.0',
				parameters: [
					{
						name: 'imageName',
						type: 'string',
						description: 'Docker image name',
						required: true,
					},
					{
						name: 'dockerfile',
						type: 'file',
						description: 'Dockerfile path',
						required: false,
						defaultValue: 'Dockerfile',
					},
					{
						name: 'context',
						type: 'directory',
						description: 'Build context',
						required: false,
						defaultValue: '.',
					},
				],
				command: 'docker build -t ${imageName} -f ${dockerfile} ${context}',
				executionType: 'docker',
				defaultConfig: { dockerfile: 'Dockerfile', context: '.' },
				tags: ['docker', 'build', 'deploy'],
			},
			{
				id: 'docker-push',
				name: 'Push Docker Image',
				description: 'Push Docker image to registry',
				category: 'deploy',
				version: '1.0.0',
				parameters: [
					{
						name: 'imageName',
						type: 'string',
						description: 'Docker image name',
						required: true,
					},
				],
				command: 'docker push ${imageName}',
				executionType: 'docker',
				defaultConfig: {},
				tags: ['docker', 'push', 'deploy'],
			},
			// Git
			{
				id: 'git-commit',
				name: 'Git Commit',
				description: 'Commit changes to Git',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'message',
						type: 'string',
						description: 'Commit message',
						required: true,
					},
				],
				command: 'git commit -m "${message}"',
				executionType: 'command',
				defaultConfig: {},
				tags: ['git', 'commit'],
			},
			{
				id: 'git-push',
				name: 'Git Push',
				description: 'Push changes to remote repository',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'branch',
						type: 'string',
						description: 'Branch to push',
						required: false,
						defaultValue: 'main',
					},
				],
				command: 'git push origin ${branch}',
				executionType: 'command',
				defaultConfig: { branch: 'main' },
				tags: ['git', 'push'],
			},
			// Custom Script
			{
				id: 'run-script',
				name: 'Run Custom Script',
				description: 'Execute a custom script or command',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'command',
						type: 'string',
						description: 'Command or script to run',
						required: true,
					},
				],
				command: '${command}',
				executionType: 'command',
				defaultConfig: {},
				tags: ['custom', 'script'],
			},
			// Spring Boot - Maven
			{
				id: 'maven-clean',
				name: 'Maven Clean',
				description: 'Clean Maven project (removes target directory)',
				category: 'build',
				version: '1.0.0',
				parameters: [],
				command: 'mvn clean',
				executionType: 'command',
				defaultConfig: {},
				tags: ['maven', 'spring-boot', 'java', 'clean'],
			},
			{
				id: 'maven-compile',
				name: 'Maven Compile',
				description: 'Compile Maven project',
				category: 'build',
				version: '1.0.0',
				parameters: [
					{
						name: 'skipTests',
						type: 'boolean',
						description: 'Skip tests during compilation',
						required: false,
						defaultValue: false,
					},
				],
				command: 'mvn compile${skipTests ? " -DskipTests" : ""}',
				executionType: 'command',
				defaultConfig: { skipTests: false },
				tags: ['maven', 'spring-boot', 'java', 'compile'],
			},
			{
				id: 'maven-test',
				name: 'Maven Test',
				description: 'Run Maven tests',
				category: 'test',
				version: '1.0.0',
				parameters: [
					{
						name: 'testProfile',
						type: 'string',
						description: 'Test profile to use',
						required: false,
						defaultValue: '',
					},
				],
				command: 'mvn test${testProfile ? ` -P${testProfile}` : ""}',
				executionType: 'command',
				defaultConfig: { testProfile: '' },
				tags: ['maven', 'spring-boot', 'java', 'test'],
			},
			{
				id: 'maven-package',
				name: 'Maven Package',
				description: 'Package Maven project (creates JAR/WAR)',
				category: 'build',
				version: '1.0.0',
				parameters: [
					{
						name: 'skipTests',
						type: 'boolean',
						description: 'Skip tests during packaging',
						required: false,
						defaultValue: false,
					},
					{
						name: 'profile',
						type: 'string',
						description: 'Maven profile to use',
						required: false,
						defaultValue: '',
					},
				],
				command: 'mvn package${skipTests ? " -DskipTests" : ""}${profile ? ` -P${profile}` : ""}',
				executionType: 'command',
				defaultConfig: { skipTests: false, profile: '' },
				tags: ['maven', 'spring-boot', 'java', 'package'],
			},
			{
				id: 'maven-install',
				name: 'Maven Install',
				description: 'Install Maven project to local repository',
				category: 'build',
				version: '1.0.0',
				parameters: [
					{
						name: 'skipTests',
						type: 'boolean',
						description: 'Skip tests during installation',
						required: false,
						defaultValue: false,
					},
				],
				command: 'mvn install${skipTests ? " -DskipTests" : ""}',
				executionType: 'command',
				defaultConfig: { skipTests: false },
				tags: ['maven', 'spring-boot', 'java', 'install'],
			},
			{
				id: 'maven-verify',
				name: 'Maven Verify',
				description: 'Run Maven integration tests and verify',
				category: 'test',
				version: '1.0.0',
				parameters: [],
				command: 'mvn verify',
				executionType: 'command',
				defaultConfig: {},
				tags: ['maven', 'spring-boot', 'java', 'verify', 'integration-test'],
			},
			// Spring Boot - Gradle
			{
				id: 'gradle-clean',
				name: 'Gradle Clean',
				description: 'Clean Gradle project',
				category: 'build',
				version: '1.0.0',
				parameters: [],
				command: './gradlew clean',
				executionType: 'command',
				defaultConfig: {},
				tags: ['gradle', 'spring-boot', 'java', 'clean'],
			},
			{
				id: 'gradle-build',
				name: 'Gradle Build',
				description: 'Build Gradle project',
				category: 'build',
				version: '1.0.0',
				parameters: [
					{
						name: 'skipTests',
						type: 'boolean',
						description: 'Skip tests during build',
						required: false,
						defaultValue: false,
					},
				],
				command: './gradlew build${skipTests ? " -x test" : ""}',
				executionType: 'command',
				defaultConfig: { skipTests: false },
				tags: ['gradle', 'spring-boot', 'java', 'build'],
			},
			{
				id: 'gradle-test',
				name: 'Gradle Test',
				description: 'Run Gradle tests',
				category: 'test',
				version: '1.0.0',
				parameters: [
					{
						name: 'testFilter',
						type: 'string',
						description: 'Test filter pattern',
						required: false,
						defaultValue: '',
					},
				],
				command: './gradlew test${testFilter ? ` --tests "${testFilter}"` : ""}',
				executionType: 'command',
				defaultConfig: { testFilter: '' },
				tags: ['gradle', 'spring-boot', 'java', 'test'],
			},
			{
				id: 'gradle-bootJar',
				name: 'Gradle BootJar',
				description: 'Create Spring Boot executable JAR',
				category: 'build',
				version: '1.0.0',
				parameters: [],
				command: './gradlew bootJar',
				executionType: 'command',
				defaultConfig: {},
				tags: ['gradle', 'spring-boot', 'java', 'package'],
			},
			// Spring Boot Specific
			{
				id: 'spring-boot-run',
				name: 'Spring Boot Run',
				description: 'Run Spring Boot application',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'buildTool',
						type: 'select',
						description: 'Build tool',
						required: true,
						defaultValue: 'maven',
						options: ['maven', 'gradle'],
					},
					{
						name: 'profile',
						type: 'string',
						description: 'Spring profile to use',
						required: false,
						defaultValue: '',
					},
				],
				command: '${buildTool === "maven" ? "mvn spring-boot:run" : "./gradlew bootRun"}${profile ? ` -Dspring.profiles.active=${profile}` : ""}',
				executionType: 'command',
				defaultConfig: { buildTool: 'maven', profile: '' },
				tags: ['spring-boot', 'java', 'run'],
			},
			{
				id: 'spring-boot-test',
				name: 'Spring Boot Test',
				description: 'Run Spring Boot integration tests',
				category: 'test',
				version: '1.0.0',
				parameters: [
					{
						name: 'buildTool',
						type: 'select',
						description: 'Build tool',
						required: true,
						defaultValue: 'maven',
						options: ['maven', 'gradle'],
					},
					{
						name: 'testProfile',
						type: 'string',
						description: 'Test profile',
						required: false,
						defaultValue: 'test',
					},
				],
				command: '${buildTool === "maven" ? "mvn test" : "./gradlew test"} -Dspring.profiles.active=${testProfile}',
				executionType: 'command',
				defaultConfig: { buildTool: 'maven', testProfile: 'test' },
				tags: ['spring-boot', 'java', 'test', 'integration'],
			},
			// Database Migration
			{
				id: 'flyway-migrate',
				name: 'Flyway Migrate',
				description: 'Run Flyway database migrations',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'buildTool',
						type: 'select',
						description: 'Build tool',
						required: true,
						defaultValue: 'maven',
						options: ['maven', 'gradle'],
					},
				],
				command: '${buildTool === "maven" ? "mvn flyway:migrate" : "./gradlew flywayMigrate"}',
				executionType: 'command',
				defaultConfig: { buildTool: 'maven' },
				tags: ['spring-boot', 'java', 'database', 'migration', 'flyway'],
			},
			{
				id: 'liquibase-update',
				name: 'Liquibase Update',
				description: 'Run Liquibase database migrations (apply pending changesets)',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'buildTool',
						type: 'select',
						description: 'Build tool',
						required: true,
						defaultValue: 'maven',
						options: ['maven', 'gradle'],
					},
					{
						name: 'contexts',
						type: 'string',
						description: 'Comma-separated list of contexts to execute',
						required: false,
						defaultValue: '',
					},
				],
				command: '${buildTool === "maven" ? "mvn liquibase:update" : "./gradlew liquibaseUpdate"}${contexts ? ` -Dliquibase.contexts=${contexts}` : ""}',
				executionType: 'command',
				defaultConfig: { buildTool: 'maven', contexts: '' },
				tags: ['spring-boot', 'java', 'database', 'migration', 'liquibase'],
			},
			{
				id: 'liquibase-status',
				name: 'Liquibase Status',
				description: 'Check Liquibase migration status (count of pending changesets)',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'buildTool',
						type: 'select',
						description: 'Build tool',
						required: true,
						defaultValue: 'maven',
						options: ['maven', 'gradle'],
					},
				],
				command: '${buildTool === "maven" ? "mvn liquibase:status" : "./gradlew liquibaseStatus"}',
				executionType: 'command',
				defaultConfig: { buildTool: 'maven' },
				tags: ['spring-boot', 'java', 'database', 'migration', 'liquibase', 'status'],
			},
			{
				id: 'liquibase-validate',
				name: 'Liquibase Validate',
				description: 'Validate Liquibase changelog file',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'buildTool',
						type: 'select',
						description: 'Build tool',
						required: true,
						defaultValue: 'maven',
						options: ['maven', 'gradle'],
					},
				],
				command: '${buildTool === "maven" ? "mvn liquibase:validate" : "./gradlew liquibaseValidate"}',
				executionType: 'command',
				defaultConfig: { buildTool: 'maven' },
				tags: ['spring-boot', 'java', 'database', 'migration', 'liquibase', 'validate'],
			},
			{
				id: 'liquibase-test',
				name: 'Liquibase Test',
				description: 'Test Liquibase migrations (update with testing rollback)',
				category: 'test',
				version: '1.0.0',
				parameters: [
					{
						name: 'buildTool',
						type: 'select',
						description: 'Build tool',
						required: true,
						defaultValue: 'maven',
						options: ['maven', 'gradle'],
					},
				],
				command: '${buildTool === "maven" ? "mvn liquibase:updateTestingRollback" : "./gradlew liquibaseUpdateTestingRollback"}',
				executionType: 'command',
				defaultConfig: { buildTool: 'maven' },
				tags: ['spring-boot', 'java', 'database', 'migration', 'liquibase', 'test'],
			},
			{
				id: 'liquibase-rollback',
				name: 'Liquibase Rollback',
				description: 'Rollback Liquibase database changes',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'buildTool',
						type: 'select',
						description: 'Build tool',
						required: true,
						defaultValue: 'maven',
						options: ['maven', 'gradle'],
					},
					{
						name: 'rollbackCount',
						type: 'number',
						description: 'Number of changesets to rollback',
						required: false,
						defaultValue: 1,
					},
					{
						name: 'rollbackTag',
						type: 'string',
						description: 'Tag to rollback to (alternative to count)',
						required: false,
						defaultValue: '',
					},
				],
				command: '${buildTool === "maven" ? "mvn liquibase:rollback" : "./gradlew liquibaseRollback"}${rollbackCount ? ` -Dliquibase.rollbackCount=${rollbackCount}` : ""}${rollbackTag ? ` -Dliquibase.rollbackTag=${rollbackTag}` : ""}',
				executionType: 'command',
				defaultConfig: { buildTool: 'maven', rollbackCount: 1, rollbackTag: '' },
				tags: ['spring-boot', 'java', 'database', 'migration', 'liquibase', 'rollback'],
			},
			{
				id: 'liquibase-clear-checksums',
				name: 'Liquibase Clear Checksums',
				description: 'Clear Liquibase checksums (useful when changelog is modified)',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'buildTool',
						type: 'select',
						description: 'Build tool',
						required: true,
						defaultValue: 'maven',
						options: ['maven', 'gradle'],
					},
				],
				command: '${buildTool === "maven" ? "mvn liquibase:clearCheckSums" : "./gradlew liquibaseClearChecksums"}',
				executionType: 'command',
				defaultConfig: { buildTool: 'maven' },
				tags: ['spring-boot', 'java', 'database', 'migration', 'liquibase', 'checksums'],
			},
			{
				id: 'liquibase-generate-changelog',
				name: 'Liquibase Generate Changelog',
				description: 'Generate Liquibase changelog from existing database',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'buildTool',
						type: 'select',
						description: 'Build tool',
						required: true,
						defaultValue: 'maven',
						options: ['maven', 'gradle'],
					},
					{
						name: 'outputFile',
						type: 'string',
						description: 'Output changelog file path',
						required: false,
						defaultValue: 'dbchangelog.xml',
					},
				],
				command: '${buildTool === "maven" ? "mvn liquibase:generateChangeLog" : "./gradlew liquibaseGenerateChangelog"}${outputFile ? ` -Dliquibase.changeLogFile=${outputFile}` : ""}',
				executionType: 'command',
				defaultConfig: { buildTool: 'maven', outputFile: 'dbchangelog.xml' },
				tags: ['spring-boot', 'java', 'database', 'migration', 'liquibase', 'generate'],
			},
			{
				id: 'liquibase-tag',
				name: 'Liquibase Tag',
				description: 'Tag the current database state with a version name',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'buildTool',
						type: 'select',
						description: 'Build tool',
						required: true,
						defaultValue: 'maven',
						options: ['maven', 'gradle'],
					},
					{
						name: 'tagName',
						type: 'string',
						description: 'Tag name to apply',
						required: true,
					},
				],
				command: '${buildTool === "maven" ? "mvn liquibase:tag" : "./gradlew liquibaseTag"} -Dliquibase.tag=${tagName}',
				executionType: 'command',
				defaultConfig: { buildTool: 'maven' },
				tags: ['spring-boot', 'java', 'database', 'migration', 'liquibase', 'tag'],
			},
			// Microservices
			{
				id: 'eureka-register',
				name: 'Register with Eureka',
				description: 'Register service with Eureka service discovery',
				category: 'deploy',
				version: '1.0.0',
				parameters: [
					{
						name: 'serviceName',
						type: 'string',
						description: 'Service name to register',
						required: true,
					},
					{
						name: 'eurekaUrl',
						type: 'string',
						description: 'Eureka server URL',
						required: true,
						defaultValue: 'http://localhost:8761/eureka',
					},
				],
				command: 'curl -X POST ${eurekaUrl}/eureka/apps/${serviceName}',
				executionType: 'command',
				defaultConfig: { eurekaUrl: 'http://localhost:8761/eureka' },
				tags: ['spring-boot', 'microservices', 'eureka', 'service-discovery'],
			},
			{
				id: 'consul-register',
				name: 'Register with Consul',
				description: 'Register service with Consul service discovery',
				category: 'deploy',
				version: '1.0.0',
				parameters: [
					{
						name: 'serviceName',
						type: 'string',
						description: 'Service name to register',
						required: true,
					},
					{
						name: 'consulUrl',
						type: 'string',
						description: 'Consul server URL',
						required: true,
						defaultValue: 'http://localhost:8500',
					},
				],
				command: 'curl -X PUT ${consulUrl}/v1/agent/service/register -d \'{"ID":"${serviceName}","Name":"${serviceName}"}\'',
				executionType: 'command',
				defaultConfig: { consulUrl: 'http://localhost:8500' },
				tags: ['spring-boot', 'microservices', 'consul', 'service-discovery'],
			},
			// Manual Approval
			{
				id: 'manual-approval',
				name: 'Manual Approval',
				description: 'Pause pipeline and wait for manual approval before continuing',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'message',
						type: 'string',
						description: 'Approval message to display',
						required: false,
						defaultValue: 'Please review and approve to continue',
					},
				],
				command: 'echo "Manual approval required: ${message}"',
				executionType: 'command',
				defaultConfig: { message: 'Please review and approve to continue' },
				tags: ['manual', 'approval', 'intervention'],
			},
			// VPN Connection
			{
				id: 'vpn-connect',
				name: 'VPN Connect',
				description: 'Connect to OpenVPN server with authentication',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'configDir',
						type: 'directory',
						description: 'Directory containing VPN config files',
						required: true,
						defaultValue: '/path/to/vpn/config',
					},
					{
						name: 'configFile',
						type: 'string',
						description: 'OpenVPN config filename (.ovpn)',
						required: true,
						defaultValue: 'gcp.ovpn',
					},
					{
						name: 'authFile',
						type: 'string',
						description: 'Auth credentials filename',
						required: false,
						defaultValue: 'auth.txt',
					},
					{
						name: 'action',
						type: 'select',
						description: 'VPN action to perform',
						required: true,
						defaultValue: 'connect',
						options: ['connect', 'disconnect', 'status'],
					},
				],
				command: 'sudo openvpn --config "${configDir}/${configFile}" --auth-retry interact --auth-user-pass "${configDir}/${authFile}"',
				executionType: 'script',
				defaultConfig: {
					configDir: '/path/to/vpn/config',
					configFile: 'gcp.ovpn',
					authFile: 'auth.txt',
					action: 'connect',
				},
				tags: ['vpn', 'openvpn', 'network', 'security', 'gcp'],
				icon: 'shield',
			},
		];
	}

	/**
	 * Get all blocks (default + custom)
	 */
	async getBlocks(): Promise<Block[]> {
		try {
			log.info('Loading blocks');
			const customBlocks = await invoke<Block[]>('get_blocks');
			return [...this.defaultBlocks, ...customBlocks];
		} catch (error) {
			log.warn('Failed to load custom blocks, using defaults only', error);
			return this.defaultBlocks;
		}
	}

	/**
	 * Get default blocks only
	 */
	getDefaultBlocks(): Block[] {
		return [...this.defaultBlocks];
	}

	/**
	 * Get blocks by category
	 */
	async getBlocksByCategory(category: Block['category']): Promise<Block[]> {
		const blocks = await this.getBlocks();
		return blocks.filter((b) => b.category === category);
	}

	/**
	 * Get blocks by tag
	 */
	async getBlocksByTag(tag: string): Promise<Block[]> {
		const blocks = await this.getBlocks();
		return blocks.filter((b) => b.tags.includes(tag));
	}

	/**
	 * Search blocks
	 */
	async searchBlocks(query: string): Promise<Block[]> {
		const blocks = await this.getBlocks();
		const lowerQuery = query.toLowerCase();
		return blocks.filter(
			(b) =>
				b.name.toLowerCase().includes(lowerQuery) ||
				b.description.toLowerCase().includes(lowerQuery) ||
				b.tags.some((tag) => tag.toLowerCase().includes(lowerQuery))
		);
	}

	/**
	 * Get a specific block by ID
	 */
	async getBlock(blockId: string): Promise<Block | null> {
		const blocks = await this.getBlocks();
		return blocks.find((b) => b.id === blockId) || null;
	}

	/**
	 * Create a custom block
	 */
	async createBlock(request: CreateBlockRequest): Promise<Block> {
		try {
			log.info('Creating block', { name: request.name });
			const block = await invoke<Block>('create_block', { request });
			log.info('Block created', { id: block.id });
			return block;
		} catch (error) {
			log.error('Failed to create block', { error });
			throw error;
		}
	}

	/**
	 * Update a custom block
	 */
	async updateBlock(blockId: string, request: Partial<CreateBlockRequest>): Promise<Block> {
		try {
			log.info('Updating block', { blockId });
			const block = await invoke<Block>('update_block', { blockId, request });
			log.info('Block updated', { blockId });
			return block;
		} catch (error) {
			log.error('Failed to update block', { error });
			throw error;
		}
	}

	/**
	 * Delete a custom block
	 */
	async deleteBlock(blockId: string): Promise<void> {
		try {
			log.info('Deleting block', { blockId });
			await invoke('delete_block', { blockId });
			log.info('Block deleted', { blockId });
		} catch (error) {
			log.error('Failed to delete block', { error });
			throw error;
		}
	}
}

export const blockLibraryService = BlockLibraryService.getInstance();

