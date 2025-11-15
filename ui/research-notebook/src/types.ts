export type NotebookShell = "web" | "desktop" | "mobile" | "cli";

export interface NotebookCitation {
  id: string;
  label: string;
  url?: string;
  publisher?: string;
  accessedAt?: string;
  snippet?: string;
}

export type NotebookMediaKind = "image" | "video" | "audio" | "document";

export interface NotebookMediaAsset {
  id: string;
  kind: NotebookMediaKind;
  uri: string;
  thumbnailUri?: string;
  description?: string;
  credit?: string;
}

export interface NotebookSection {
  id: string;
  heading: string;
  summary: string;
  body: string[];
  citations?: NotebookCitation[];
  media?: NotebookMediaAsset[];
  actions?: {
    label: string;
    workflowId: string;
    stageId?: string;
  }[];
}

export interface ResearchNotebookMetadata {
  id: string;
  title: string;
  summary: string;
  persona?: string;
  tags?: string[];
  shells: NotebookShell[];
  createdAt?: string;
  updatedAt?: string;
  author?: string;
  reviewers?: string[];
  citations?: NotebookCitation[];
}

export interface ResearchNotebookRecord {
  metadata: ResearchNotebookMetadata;
  sections: NotebookSection[];
  appendix?: NotebookSection[];
}

export interface NotebookToPageOptions {
  surface?: "web" | "desktop" | "mobile";
}
