import React, { useState } from 'react';
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
  Download, 
  Upload, 
  Trash2, 
  FileCode,
  Clock,
} from 'lucide-react';
import { v4 as uuidv4 } from 'uuid';

export const FileManager: React.FC = () => {
  const {
    files,
    currentFile,
    setCurrentFile,
    addFile,
    deleteFile,
    editorContent,
  } = useEDSLStore();

  const [isNewFileDialogOpen, setIsNewFileDialogOpen] = useState(false);
  const [newFileName, setNewFileName] = useState('');
  const [newFileContent, setNewFileContent] = useState('');
  const [fileInputRef, setFileInputRef] = useState<HTMLInputElement | null>(null);

  const handleCreateFile = () => {
    if (!newFileName.trim()) return;
    
    const fileName = newFileName.endsWith('.edsl') ? newFileName : `${newFileName}.edsl`;
    const newFile: EDSLFile = {
      name: fileName,
      content: newFileContent || `---
layout: dagre
---

# New diagram
node1[Node 1] -> node2[Node 2]`,
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

  const handleDeleteFile = (file: EDSLFile, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm(`Are you sure you want to delete "${file.name}"?`)) {
      deleteFile(file.name);
    }
  };

  const handleImportFile = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (event) => {
      const content = event.target?.result as string;
      const edslFile: EDSLFile = {
        name: file.name,
        content,
        lastModified: new Date(),
      };
      addFile(edslFile);
      setCurrentFile(edslFile);
    };
    reader.readAsText(file);
    
    // Reset file input
    if (fileInputRef) {
      fileInputRef.value = '';
    }
  };

  const handleExportFile = (file: EDSLFile) => {
    const blob = new Blob([file.content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = file.name;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const handleExportCurrent = () => {
    if (!currentFile) return;
    
    const updatedFile = { ...currentFile, content: editorContent };
    handleExportFile(updatedFile);
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
    <div className="h-full flex flex-col bg-gray-50 border-r">
      {/* Header */}
      <div className="p-4 border-b bg-white">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-lg font-semibold flex items-center">
            <FileCode className="h-5 w-5 mr-2" />
            Files
          </h2>
        </div>
        
        <div className="flex space-x-2">
          <Dialog open={isNewFileDialogOpen} onOpenChange={setIsNewFileDialogOpen}>
            <DialogTrigger asChild>
              <Button size="sm" className="flex-1">
                <Plus className="h-4 w-4 mr-1" />
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
          
          <Button
            size="sm"
            variant="outline"
            onClick={() => fileInputRef?.click()}
            title="Import EDSL file"
          >
            <Upload className="h-4 w-4" />
          </Button>
          
          <Button
            size="sm"
            variant="outline"
            onClick={handleExportCurrent}
            disabled={!currentFile}
            title="Export current file"
          >
            <Download className="h-4 w-4" />
          </Button>
        </div>
        
        <input
          type="file"
          ref={setFileInputRef}
          onChange={handleImportFile}
          accept=".edsl,.txt"
          style={{ display: 'none' }}
        />
      </div>

      {/* File List */}
      <div className="flex-1 overflow-y-auto">
        {files.length === 0 ? (
          <div className="p-4 text-center text-gray-500">
            <FileText className="h-12 w-12 mx-auto mb-3 text-gray-300" />
            <p className="text-sm">No files yet</p>
            <p className="text-xs text-gray-400 mt-1">
              Create a new file to get started
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
                <div className="flex items-center justify-between">
                  <div className="flex items-center min-w-0 flex-1">
                    <FileText className="h-4 w-4 text-blue-600 mr-2 flex-shrink-0" />
                    <div className="min-w-0 flex-1">
                      <p className="text-sm font-medium text-gray-900 truncate">
                        {file.name}
                      </p>
                      <div className="flex items-center text-xs text-gray-500 mt-1">
                        <Clock className="h-3 w-3 mr-1" />
                        {formatDate(file.lastModified)}
                      </div>
                    </div>
                  </div>
                  
                  <div className="flex items-center space-x-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleExportFile(file);
                      }}
                      title="Export file"
                      className="h-6 w-6 p-0"
                    >
                      <Download className="h-3 w-3" />
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={(e) => handleDeleteFile(file, e)}
                      title="Delete file"
                      className="h-6 w-6 p-0 text-red-600 hover:text-red-700 hover:bg-red-50"
                    >
                      <Trash2 className="h-3 w-3" />
                    </Button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};