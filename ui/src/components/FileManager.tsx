import React, { useState, useEffect } from 'react';
import { useEDSLStore } from '../store/edsl-store';
import { EDSLFile } from '../types/edsl';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from './ui/dialog';
import { Label } from './ui/label';
import { Textarea } from './ui/textarea';
import {
  FileText,
  Plus,
  FileCode,
  Clock,
  FolderOpen,
  Loader2,
} from 'lucide-react';
import { fileService } from '../services/file-service';
import { Alert, AlertDescription } from './ui/alert';

export const FileManager: React.FC = () => {
  const {
    files,
    currentFile,
    setCurrentFile,
    addFile,
    editorContent,
    updateFileContent,
  } = useEDSLStore();

  const [isNewFileDialogOpen, setIsNewFileDialogOpen] = useState(false);
  const [newFileName, setNewFileName] = useState('');
  const [newFileContent, setNewFileContent] = useState('');
  const [serverPath, setServerPath] = useState<string>('');
  const [isLoadingFiles, setIsLoadingFiles] = useState(false);
  const [serverError, setServerError] = useState<string | null>(null);
  const [folderInputRef, setFolderInputRef] = useState<HTMLInputElement | null>(null);

  // Update file content when editor changes (for local files)
  useEffect(() => {
    if (!currentFile) return;

    // Update the file content in store when editor content changes
    if (editorContent !== currentFile.content) {
      updateFileContent(currentFile.name, editorContent);
    }
  }, [editorContent, currentFile, updateFileContent]);

  const handleCreateFile = () => {
    if (!newFileName.trim()) return;

    const fileName = newFileName.endsWith('.edsl') ? newFileName : `${newFileName}.edsl`;
    const content = newFileContent || `---
layout: dagre
---

# New diagram
node1[Node 1] -> node2[Node 2]`;

    // Add to local state
    const newFile: EDSLFile = {
      name: fileName,
      content,
      lastModified: new Date(),
    };

    addFile(newFile);
    setCurrentFile(newFile);
    setIsNewFileDialogOpen(false);
    setNewFileName('');
    setNewFileContent('');
  };

  const handleFileSelect = (file: EDSLFile) => {
    setCurrentFile(file);
  };

  const handleFolderSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files || files.length === 0) return;

    setIsLoadingFiles(true);
    setServerError(null);

    try {
      // Get the absolute folder path from the first file
      const firstFile = files[0];
      let folderPath = '';

      if (firstFile.webkitRelativePath) {
        // Get the folder path by removing the filename
        const pathParts = firstFile.webkitRelativePath.split('/');
        pathParts.pop(); // Remove filename
        folderPath = pathParts.join('/');
      }

      // If we can't get the path from webkitRelativePath, use a default
      if (!folderPath) {
        folderPath = 'examples';
      }

      console.log('Using folder path:', folderPath);

      // Use the server API to get file list from the folder path
      const serverFiles = await fileService.loadFilesFromDirectory(folderPath);

      if (serverFiles.length > 0) {
        // Clear existing files
        useEDSLStore.getState().clearFiles();

        // Add all loaded files
        serverFiles.forEach(file => {
          addFile(file);
        });

        // Set the first file as current
        setCurrentFile(serverFiles[0]);
        setServerPath(folderPath);
      } else {
        setServerError('No .edsl files found in the selected directory');
      }
    } catch (error) {
      setServerError(error instanceof Error ? error.message : 'Failed to load files from server');
      console.error('Failed to load server files:', error);
    } finally {
      setIsLoadingFiles(false);
    }

    // Reset folder input
    if (folderInputRef) {
      folderInputRef.value = '';
    }
  };

  const formatDate = (date: Date) => {
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }).format(date);
  };

  return (
    <div className="flex flex-col h-full bg-gray-50 border-r">
      {/* Header */}
      <div className="p-4 bg-white border-b">
        <div className="flex justify-between items-center mb-3">
          <h2 className="flex items-center text-lg font-semibold">
            <FileCode className="mr-2 w-5 h-5" />
            Files
          </h2>
        </div>

        {serverPath && (
          <div className="px-2 py-1 mb-2 text-xs text-gray-500 bg-gray-100 rounded">
            <span className="font-medium">Folder:</span> {serverPath}
          </div>
        )}

        {serverError && (
          <Alert variant="destructive" className="mb-2">
            <AlertDescription>{serverError}</AlertDescription>
          </Alert>
        )}

        <div className="flex gap-2">
          {/* Open Folder Button */}
          <Button
            size="sm"
            variant="outline"
            className="flex-1"
            onClick={() => folderInputRef?.click()}
            disabled={isLoadingFiles}
          >
            {isLoadingFiles ? (
              <>
                <Loader2 className="mr-1 w-4 h-4 animate-spin" />
                Loading...
              </>
            ) : (
              <>
                <FolderOpen className="mr-1 w-4 h-4" />
                Open Folder
              </>
            )}
          </Button>

          {/* New File Dialog */}
          <Dialog open={isNewFileDialogOpen} onOpenChange={setIsNewFileDialogOpen}>
            <DialogTrigger asChild>
              <Button size="sm" className="flex-1">
                <Plus className="mr-1 w-4 h-4" />
                New
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Create New EDSL File</DialogTitle>
              </DialogHeader>
              <div className="space-y-4">
                <div>
                  <Label htmlFor="fileName">File Name</Label>
                  <Input
                    id="fileName"
                    value={newFileName}
                    onChange={(e) => setNewFileName(e.target.value)}
                    placeholder="diagram.edsl"
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') {
                        handleCreateFile();
                      }
                    }}
                  />
                </div>
                <div>
                  <Label htmlFor="fileContent">Initial Content (Optional)</Label>
                  <Textarea
                    id="fileContent"
                    value={newFileContent}
                    onChange={(e) => setNewFileContent(e.target.value)}
                    placeholder="Leave empty for default template"
                    rows={8}
                  />
                </div>
                <div className="flex justify-end space-x-2">
                  <Button
                    variant="outline"
                    onClick={() => setIsNewFileDialogOpen(false)}
                  >
                    Cancel
                  </Button>
                  <Button onClick={handleCreateFile} disabled={!newFileName.trim()}>
                    Create File
                  </Button>
                </div>
              </div>
            </DialogContent>
          </Dialog>
        </div>
      </div>

      {/* File List */}
      <div className="overflow-y-auto flex-1">
        {files.length === 0 ? (
          <div className="p-4 text-center text-gray-500">
            <FileText className="mx-auto mb-3 w-12 h-12 text-gray-300" />
            <p className="text-sm">No files yet</p>
            <p className="mt-1 text-xs text-gray-400">
              Open a folder to get started
            </p>
          </div>
        ) : (
          <div className="p-2">
            {files.map((file) => (
              <div
                key={file.name}
                className={`
                  p-3 rounded-lg cursor-pointer group transition-colors mb-2
                  ${currentFile?.name === file.name
                    ? 'bg-blue-100 border border-blue-200'
                    : 'bg-white hover:bg-gray-50 border border-transparent hover:border-gray-200'
                  }
                `}
                onClick={() => handleFileSelect(file)}
              >
                <div className="flex justify-between items-center">
                  <div className="flex flex-1 items-center min-w-0">
                    <FileText className="flex-shrink-0 mr-2 w-4 h-4 text-blue-600" />
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-gray-900 truncate">
                        {file.name}
                      </p>
                      <div className="flex items-center mt-1 text-xs text-gray-500">
                        <Clock className="mr-1 w-3 h-3" />
                        {formatDate(file.lastModified)}
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Hidden folder input */}
      <input
        type="file"
        ref={setFolderInputRef}
        onChange={handleFolderSelect}
        webkitdirectory=""
        directory=""
        accept=".edsl"
        style={{ display: 'none' }}
      />
    </div>
  );
};
