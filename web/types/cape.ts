export interface CapeRow {
  id: string;
  name: string;
  url: string;
  file_sha1: string;
  size: number;
  uploaded_by?: string | null;
  uploaded_at: string;
}
