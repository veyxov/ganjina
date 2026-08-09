export interface Asset {
  id: string;
  hash: string;
  original_filename: string;
  size_bytes: number;
  content_type: string;
  thumbnail_hash: string | null;
  owner_id: string | null;
  width: number | null;
  height: number | null;
  created_at: string;
}

export interface Collection {
  id: string;
  name: string;
  created_at: string;
}

export interface Owner {
  id: string;
  name: string;
}

export interface PhotoMetadata {
  taken_at_local: string | null;
  taken_at_offset_minutes: number | null;
  camera: string | null;
  gps_lat: number | null;
  gps_lon: number | null;
}

export interface AdjacentAssets {
  prev_id: string | null;
  next_id: string | null;
}

export interface StatusSummary {
  failed_count: number;
}
