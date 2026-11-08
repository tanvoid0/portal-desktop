export interface ElevationStatus {
  isElevated: boolean;
  platform: string;
}

export interface ElevatedRunResult {
  success: boolean;
  cancelled: boolean;
  message: string;
}
