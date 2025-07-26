import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import { EDSLFile, CompilerOptions, EDSLCompilationResult, EDSLValidationResult } from '../types/edsl';

interface EDSLStore {
  // File management
  currentFile: EDSLFile | null;
  files: EDSLFile[];
  setCurrentFile: (file: EDSLFile | null) => void;
  addFile: (file: EDSLFile) => void;
  updateFileContent: (name: string, content: string) => void;
  deleteFile: (name: string) => void;
  clearFiles: () => void;
  
  // Editor state
  editorContent: string;
  setEditorContent: (content: string) => void;
  
  // Compiler options
  compilerOptions: CompilerOptions;
  setCompilerOptions: (options: Partial<CompilerOptions>) => void;
  
  // Compilation results
  compilationResult: EDSLCompilationResult | null;
  validationResult: EDSLValidationResult | null;
  isCompiling: boolean;
  isValidating: boolean;
  setCompilationResult: (result: EDSLCompilationResult | null) => void;
  setValidationResult: (result: EDSLValidationResult | null) => void;
  setIsCompiling: (isCompiling: boolean) => void;
  setIsValidating: (isValidating: boolean) => void;
  
  // UI state
  showPreview: boolean;
  sidebarWidth: number;
  setShowPreview: (show: boolean) => void;
  setSidebarWidth: (width: number) => void;
}

export const useEDSLStore = create<EDSLStore>()(
  devtools(
    (set, get) => ({
      // File management
      currentFile: null,
      files: [],
      setCurrentFile: (file) => {
        set({ 
          currentFile: file,
          editorContent: file ? file.content : '',
          // Clear validation and compilation results when switching files
          validationResult: null,
          compilationResult: null,
        });
      },
      addFile: (file) => {
        set((state) => ({
          files: [...state.files, file],
        }));
      },
      updateFileContent: (name, content) => {
        set((state) => ({
          files: state.files.map(f => 
            f.name === name 
              ? { ...f, content, lastModified: new Date() }
              : f
          ),
          currentFile: state.currentFile?.name === name 
            ? { ...state.currentFile, content, lastModified: new Date() }
            : state.currentFile,
        }));
      },
      deleteFile: (name) => {
        set((state) => ({
          files: state.files.filter(f => f.name !== name),
          currentFile: state.currentFile?.name === name ? null : state.currentFile,
        }));
      },
      clearFiles: () => {
        set({
          files: [],
          currentFile: null,
          editorContent: '',
        });
      },
      
      // Editor state
      editorContent: `---
layout: dagre
theme: light
---

web[Web Server]
api[API Gateway] {
  shape: rectangle;
  strokeColor: "#007acc";
}
db[Database] {
  shape: cylinder;
  backgroundColor: "#f0f0f0";
}

web -> api -> db`,
      setEditorContent: (content) => set({ editorContent: content }),
      
      // Compiler options
      compilerOptions: {
        layout: 'dagre',
        validate: true,
        verbose: false,
      },
      setCompilerOptions: (options) => {
        set((state) => ({
          compilerOptions: { ...state.compilerOptions, ...options },
        }));
      },
      
      // Compilation results
      compilationResult: null,
      validationResult: null,
      isCompiling: false,
      isValidating: false,
      setCompilationResult: (result) => set({ compilationResult: result }),
      setValidationResult: (result) => set({ validationResult: result }),
      setIsCompiling: (isCompiling) => set({ isCompiling }),
      setIsValidating: (isValidating) => set({ isValidating }),
      
      // UI state
      showPreview: true,
      sidebarWidth: 300,
      setShowPreview: (show) => set({ showPreview: show }),
      setSidebarWidth: (width) => set({ sidebarWidth: width }),
    }),
    {
      name: 'edsl-store',
    }
  )
);