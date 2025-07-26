import React from 'react';
import { useEDSLStore } from '../store/edsl-store';
import { LAYOUT_ALGORITHMS } from '../types/edsl';
import { Button } from './ui/button';
import { Label } from './ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { Switch } from './ui/switch';
import { Separator } from './ui/separator';
import { Badge } from './ui/badge';
import { 
  Settings, 
  Layout, 
  CheckCircle, 
  Eye, 
  EyeOff,
  GitBranch,
  Zap,
} from 'lucide-react';

export const SettingsPanel: React.FC = () => {
  const {
    compilerOptions,
    setCompilerOptions,
    showPreview,
    setShowPreview,
    compilationResult,
    validationResult,
    isCompiling,
    isValidating,
  } = useEDSLStore();

  const handleLayoutChange = (layout: string) => {
    setCompilerOptions({ layout });
  };

  const handleValidationToggle = (validate: boolean) => {
    setCompilerOptions({ validate });
  };

  const handleVerboseToggle = (verbose: boolean) => {
    setCompilerOptions({ verbose });
  };

  return (
    <div className="h-full flex flex-col bg-gray-50 border-l">
      {/* Header */}
      <div className="p-4 border-b bg-white">
        <h2 className="text-lg font-semibold flex items-center">
          <Settings className="h-5 w-5 mr-2" />
          Settings
        </h2>
      </div>

      {/* Settings Content */}
      <div className="flex-1 overflow-y-auto p-4 space-y-6">
        
        {/* Compiler Options */}
        <div className="space-y-4">
          <h3 className="text-sm font-semibold text-gray-900 flex items-center">
            <Zap className="h-4 w-4 mr-2" />
            Compiler Options
          </h3>
          
          <div className="space-y-3">
            <div>
              <Label htmlFor="layout-select" className="text-sm font-medium">
                Layout Algorithm
              </Label>
              <Select value={compilerOptions.layout} onValueChange={handleLayoutChange}>
                <SelectTrigger id="layout-select" className="w-full mt-1">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {LAYOUT_ALGORITHMS.map((algorithm) => (
                    <SelectItem key={algorithm.id} value={algorithm.id}>
                      <div>
                        <div className="font-medium">{algorithm.name}</div>
                        <div className="text-xs text-gray-500">{algorithm.description}</div>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label htmlFor="validation-switch" className="text-sm font-medium">
                  Auto Validation
                </Label>
                <p className="text-xs text-gray-500">
                  Validate EDSL syntax as you type
                </p>
              </div>
              <Switch
                id="validation-switch"
                checked={compilerOptions.validate}
                onCheckedChange={handleValidationToggle}
              />
            </div>

            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label htmlFor="verbose-switch" className="text-sm font-medium">
                  Verbose Output
                </Label>
                <p className="text-xs text-gray-500">
                  Show detailed compilation information
                </p>
              </div>
              <Switch
                id="verbose-switch"
                checked={compilerOptions.verbose}
                onCheckedChange={handleVerboseToggle}
              />
            </div>
          </div>
        </div>

        <Separator />

        {/* UI Options */}
        <div className="space-y-4">
          <h3 className="text-sm font-semibold text-gray-900 flex items-center">
            <Layout className="h-4 w-4 mr-2" />
            Display Options
          </h3>
          
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="preview-switch" className="text-sm font-medium">
                Show Preview
              </Label>
              <p className="text-xs text-gray-500">
                Display live Excalidraw preview
              </p>
            </div>
            <Switch
              id="preview-switch"
              checked={showPreview}
              onCheckedChange={setShowPreview}
            />
          </div>
        </div>

        <Separator />

        {/* Status Information */}
        <div className="space-y-4">
          <h3 className="text-sm font-semibold text-gray-900 flex items-center">
            <GitBranch className="h-4 w-4 mr-2" />
            Status
          </h3>
          
          <div className="space-y-3">
            {/* Validation Status */}
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Validation</span>
              {isValidating ? (
                <Badge variant="secondary" className="animate-pulse">
                  Validating...
                </Badge>
              ) : validationResult?.isValid ? (
                <Badge className="bg-green-100 text-green-800 border-green-200">
                  <CheckCircle className="h-3 w-3 mr-1" />
                  Valid
                </Badge>
              ) : validationResult?.isValid === false ? (
                <Badge variant="destructive">
                  Error
                </Badge>
              ) : (
                <Badge variant="secondary">
                  Not checked
                </Badge>
              )}
            </div>

            {/* Compilation Status */}
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Compilation</span>
              {isCompiling ? (
                <Badge variant="secondary" className="animate-pulse">
                  Compiling...
                </Badge>
              ) : compilationResult?.success ? (
                <Badge className="bg-green-100 text-green-800 border-green-200">
                  <CheckCircle className="h-3 w-3 mr-1" />
                  Success
                </Badge>
              ) : compilationResult?.success === false ? (
                <Badge variant="destructive">
                  Failed
                </Badge>
              ) : (
                <Badge variant="secondary">
                  Not compiled
                </Badge>
              )}
            </div>

            {/* Preview Status */}
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Preview</span>
              <Badge variant={showPreview ? "default" : "secondary"}>
                {showPreview ? (
                  <>
                    <Eye className="h-3 w-3 mr-1" />
                    Visible
                  </>
                ) : (
                  <>
                    <EyeOff className="h-3 w-3 mr-1" />
                    Hidden
                  </>
                )}
              </Badge>
            </div>
          </div>
        </div>

        {/* Error Details */}
        {(validationResult?.error || compilationResult?.error) && (
          <>
            <Separator />
            <div className="space-y-2">
              <h3 className="text-sm font-semibold text-red-600">Error Details</h3>
              <div className="bg-red-50 border border-red-200 rounded-lg p-3">
                <p className="text-sm text-red-800">
                  {validationResult?.error || compilationResult?.error}
                </p>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
};