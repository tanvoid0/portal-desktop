export enum ResourceType {
	DOCUMENT = 'document',
	PROJECT = 'project',
	DEPLOYMENT = 'deployment',
	PIPELINE = 'pipeline',
}

export const RESOURCE_TYPE_OPTIONS = [
	{ value: ResourceType.DOCUMENT, label: 'Document' },
	{ value: ResourceType.PROJECT, label: 'Project' },
	{ value: ResourceType.DEPLOYMENT, label: 'Deployment' },
	{ value: ResourceType.PIPELINE, label: 'Pipeline' },
] as const;

