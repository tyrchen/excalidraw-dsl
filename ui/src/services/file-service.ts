import { EDSLFile } from '../types/edsl';

export interface FileInfo {
  name: string;
  path: string;
  size: number;
  modified: number;
}

export interface FileListResponse {
  success: boolean;
  files?: FileInfo[];
  error?: string;
}

export interface FileContentResponse {
  success: boolean;
  content?: string;
  error?: string;
}

export class FileService {
  private baseUrl: string;

  constructor(baseUrl: string = 'http://localhost:3002') {
    this.baseUrl = baseUrl;
  }

  async listFiles(directoryPath: string): Promise<FileListResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/api/files?path=${encodeURIComponent(directoryPath)}`);
      return await response.json();
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Failed to list files',
      };
    }
  }

  async getFileContent(filePath: string): Promise<FileContentResponse> {
    try {
      // For the single-segment route, we need to encode the path differently
      // Since the route is /api/file/{path}, we need to pass the full path as one segment
      const encodedPath = encodeURIComponent(filePath);
      const response = await fetch(`${this.baseUrl}/api/file/${encodedPath}`);
      return await response.json();
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Failed to get file content',
      };
    }
  }

  async loadFilesFromDirectory(directoryPath: string): Promise<EDSLFile[]> {
    const listResponse = await this.listFiles(directoryPath);
    
    if (!listResponse.success || !listResponse.files) {
      throw new Error(listResponse.error || 'Failed to list files');
    }

    const edslFiles: EDSLFile[] = [];

    for (const fileInfo of listResponse.files) {
      const contentResponse = await this.getFileContent(fileInfo.path);
      
      if (contentResponse.success && contentResponse.content) {
        edslFiles.push({
          name: fileInfo.name,
          content: contentResponse.content,
          lastModified: new Date(fileInfo.modified * 1000),
        });
      }
    }

    return edslFiles;
  }

  async saveFile(filePath: string, content: string): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/api/file/save`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          path: filePath,
          content,
        }),
      });
      
      const result = await response.json();
      
      if (!result.success) {
        throw new Error(result.error || 'Failed to save file');
      }
    } catch (error) {
      throw new Error(error instanceof Error ? error.message : 'Failed to save file');
    }
  }
}

export const fileService = new FileService();