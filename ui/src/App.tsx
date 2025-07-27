import React, { useState, useEffect } from 'react';
import { useEDSLStore } from './store/edsl-store';
import { EdslEditor } from './components/EdslEditor';
import { ExcalidrawPreview } from './components/ExcalidrawPreview';
import { FileManager } from './components/FileManager';
import { SettingsPanel } from './components/SettingsPanel';
import { Button } from './components/ui/button';
import { PanelLeftOpen, PanelLeftClose, Settings, Eye, EyeOff } from 'lucide-react';

function App() {
  const { showPreview, currentFile, editorContent } = useEDSLStore();
  const [showFileManager, setShowFileManager] = useState(true);
  const [showSettings, setShowSettings] = useState(false);

  // Load default directory on startup
  useEffect(() => {
    const { files, addFile, setCurrentFile, clearFiles } = useEDSLStore.getState();
    
    const loadDefaultDirectory = async () => {
      try {
        // Get default directory from server
        const { fileService } = await import('./services/file-service');
        const states = await fileService.getStates();
        
        // Load files from the default directory
        const serverFiles = await fileService.loadFilesFromDirectory(states.directory);
        
        if (serverFiles.length > 0) {
          // Clear existing files and load from server
          clearFiles();
          serverFiles.forEach(file => {
            addFile(file);
          });
          setCurrentFile(serverFiles[0]);
        } else {
          // Fallback to default file if no server files found
          if (files.length === 0) {
            const defaultFile = {
              name: 'example.edsl',
              content: `---
layout: dagre
---

# Example EDSL Diagram
start[Start] -> process[Process Data]
process -> decision{Decision?}
decision -> yes[Yes Path] -> end[End]
decision -> no[No Path] -> process`,
              lastModified: new Date(),
            };
            
            addFile(defaultFile);
            setCurrentFile(defaultFile);
          }
        }
      } catch (error) {
        console.error('Failed to load default directory:', error);
        
        // Fallback to default file on error
        if (files.length === 0) {
          const defaultFile = {
            name: 'example.edsl',
            content: `---
layout: dagre
---

# Example EDSL Diagram
start[Start] -> process[Process Data]
process -> decision{Decision?}
decision -> yes[Yes Path] -> end[End]
decision -> no[No Path] -> process`,
            lastModified: new Date(),
          };
          
          addFile(defaultFile);
          setCurrentFile(defaultFile);
        }
      }
    };

    loadDefaultDirectory();
  }, []);

  return (
    <div className="h-screen bg-gray-100 flex flex-col overflow-hidden">
      {/* Header */}
      <div className="h-14 bg-white border-b flex items-center justify-between px-4 flex-shrink-0">
        <div className="flex items-center space-x-4">
          <h1 className="text-xl font-bold text-gray-900">
            ExcaliDraw DSL Editor
          </h1>
          {currentFile && (
            <span className="text-sm text-gray-500">
              {currentFile.name}
            </span>
          )}
        </div>
        
        <div className="flex items-center space-x-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowFileManager(!showFileManager)}
            title={showFileManager ? 'Hide file manager' : 'Show file manager'}
          >
            {showFileManager ? <PanelLeftClose className="h-4 w-4" /> : <PanelLeftOpen className="h-4 w-4" />}
          </Button>
          
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowSettings(!showSettings)}
            title={showSettings ? 'Hide settings' : 'Show settings'}
          >
            <Settings className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* File Manager Sidebar */}
        {showFileManager && (
          <div className="w-80 flex-shrink-0">
            <FileManager />
          </div>
        )}

        {/* Editor and Preview */}
        <div className="flex-1 flex overflow-hidden">
          {/* Editor */}
          <div className={`${showPreview ? 'w-2/5' : 'w-full'} flex flex-col border-r`}>
            <div className="h-full bg-white">
              <EdslEditor />
            </div>
          </div>

          {/* Preview */}
          {showPreview && (
            <div className="w-3/5 flex flex-col bg-white">
              <div className="h-12 border-b flex items-center justify-between px-4 bg-gray-50">
                <h2 className="text-sm font-semibold text-gray-700">Preview</h2>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => useEDSLStore.getState().setShowPreview(false)}
                  title="Hide preview"
                >
                  <EyeOff className="h-4 w-4" />
                </Button>
              </div>
              <div className="flex-1 overflow-hidden">
                <ExcalidrawPreview />
              </div>
            </div>
          )}
        </div>

        {/* Settings Sidebar */}
        {showSettings && (
          <div className="w-80 flex-shrink-0">
            <SettingsPanel />
          </div>
        )}
      </div>

      {/* Status Bar */}
      <div className="h-8 bg-gray-800 text-white text-xs flex items-center justify-between px-4 flex-shrink-0">
        <div className="flex items-center space-x-4">
          <span>EDSL Editor v1.0</span>
          {currentFile && (
            <span>{editorContent.split('\n').length} lines</span>
          )}
        </div>
        
        <div className="flex items-center space-x-4">
          {!showPreview && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => useEDSLStore.getState().setShowPreview(true)}
              className="h-6 text-xs text-white hover:text-gray-300"
            >
              <Eye className="h-3 w-3 mr-1" />
              Show Preview
            </Button>
          )}
          <span>Ready</span>
        </div>
      </div>
    </div>
  );
}

export default App;
