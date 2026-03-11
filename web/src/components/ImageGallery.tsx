import { files as filesApi } from '../api';
import type { FileStorage } from '../api/types';

interface ImageGalleryProps {
  files: FileStorage[];
  /** Maximum width for each image (default 400px). */
  maxWidth?: string;
  /** Maximum height for each image (default none). */
  maxHeight?: string;
}

/**
 * Renders a flex-wrap gallery of images from FileStorage entries.
 * Each image links to the full-size version and displays the role + size beneath.
 */
export default function ImageGallery({
  files,
  maxWidth = '400px',
  maxHeight,
}: ImageGalleryProps) {
  const images = files.filter((f) => f.mime_type.startsWith('image/'));

  if (images.length === 0) return null;

  return (
    <div
      className="image-gallery"
      style={{ display: 'flex', flexWrap: 'wrap', gap: '16px', marginBottom: '24px' }}
    >
      {images.map((f) => (
        <div key={f.id} className="image-preview" style={{ textAlign: 'center' }}>
          <a href={filesApi.getFileUrl(f.id)} target="_blank" rel="noopener noreferrer">
            <img
              src={filesApi.getFileUrl(f.id)}
              alt={f.file_name}
              style={{
                maxWidth,
                ...(maxHeight ? { maxHeight } : {}),
                borderRadius: '8px',
                border: '1px solid #ddd',
                objectFit: 'contain',
              }}
            />
          </a>
          <div style={{ marginTop: '4px', fontSize: '0.85em', color: '#666' }}>
            <span style={{ textTransform: 'capitalize' }}>{f.role}</span>
            {' \u00b7 '}
            {(f.file_size / 1024).toFixed(1)} KB
          </div>
        </div>
      ))}
    </div>
  );
}
