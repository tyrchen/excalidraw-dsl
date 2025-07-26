export interface EDSLCompilationResult {
  success: boolean;
  data?: any; // Excalidraw elements
  error?: string;
}

export interface EDSLValidationResult {
  isValid: boolean;
  error?: string;
}

export interface EDSLFile {
  name: string;
  content: string;
  lastModified: Date;
}

export interface LayoutAlgorithm {
  id: string;
  name: string;
  description: string;
}

export const LAYOUT_ALGORITHMS: LayoutAlgorithm[] = [
  {
    id: 'dagre',
    name: 'Dagre (Hierarchical)',
    description: 'Hierarchical layout with directed flow'
  },
  {
    id: 'force',
    name: 'Force-directed',
    description: 'Physics-based node positioning'
  }
];

export interface CompilerOptions {
  layout: string;
  validate: boolean;
  verbose: boolean;
}