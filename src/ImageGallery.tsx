import React, { useState, useEffect, useRef } from 'react';
import './ImageGallery.css';
import { invoke } from '@tauri-apps/api/core';

// const imageSources = [
//   'https://www.dmoe.cc/random.php',
//   'https://picsum.photos/200/300',
//   'https://www.dmoe.cc/random.php',
//   'https://picsum.photos/600/300',
//   'https://www.dmoe.cc/random.php',
//   'https://picsum.photos/1920/1080',
//   'https://www.dmoe.cc/random.php',
//   'https://picsum.photos/700/300',
//   'https://www.dmoe.cc/random.php',
//   'https://picsum.photos/1200/300',
// ];

interface Image {
  src: string;
  width: number;
  height: number;
}

const ImageGallery: React.FC = () => {
  const [layout, setLayout] = useState<'grid' | 'loose'>('loose');
  const [images, setImages] = useState<Image[]>([]);
  const galleryContainerRef = useRef<HTMLDivElement>(null);
  const [containerWidth, setContainerWidth] = useState(0);

  useEffect(() => {
    const loadImageDimensions = async () => {
      let paths = await invoke<string[]>('get_image_paths');
      console.log(`从 Rust 获得了 ${paths.length} 张图片`);
      
      const loadedImages: Image[] = await Promise.all(
        paths.map(async (src) => {
          const img = new Image();
          img.src = `http://asset.localhost/${src}`;
          await new Promise<void>((resolve) => {
            img.onload = () => resolve();
          });
          console.log(`Loaded image ${src} with dimensions: ${img.width}x${img.height}`);
          return img;
        })
      );
      setImages(loadedImages);
    };
    loadImageDimensions();
  }, []);

  useEffect(() => {
    const observer = new ResizeObserver(entries => {
      if (entries[0]) {
        setContainerWidth(entries[0].contentRect.width);
      }
    });

    const currentRef = galleryContainerRef.current;
    if (currentRef) {
      observer.observe(currentRef);
      // Set initial width
      setContainerWidth(currentRef.clientWidth);
    }

    return () => {
      if (currentRef) {
        observer.unobserve(currentRef);
      }
    };
  }, []);

  const renderLooseLayout = (width: number) => {
    if (images.length === 0 || width === 0) return null;

    const rows: Image[][] = [];
    let currentRow: Image[] = [];
    let currentRowWidth = 0;
    const maxRowWidth = width; // Use dynamic width

    images.forEach(image => {
      const aspectRatio = image.width / image.height;
      // A base height of 250px for estimation seems to work well
      const estimatedWidth = 250 * aspectRatio; 

      if (currentRowWidth + estimatedWidth > maxRowWidth && currentRow.length > 0) {
        rows.push(currentRow);
        currentRow = [image];
        currentRowWidth = estimatedWidth;
      } else {
        currentRow.push(image);
        currentRowWidth += estimatedWidth;
      }
    });
    if (currentRow.length > 0) {
      rows.push(currentRow);
    }

    return (
      <div className="image-gallery loose">
        {rows.map((row, rowIndex) => {
          const totalAspectRatio = row.reduce((acc, img) => acc + img.width / img.height, 0);
          const rowHeight = (maxRowWidth - (row.length - 1) * 6) / totalAspectRatio;

          return (
            <div key={rowIndex} className="image-row" style={{ marginBottom: '0px' }}>
              {row.map((image, imgIndex) => {
                const imageWidth = rowHeight * (image.width / image.height);
                return (
                  <div key={imgIndex} className="image-item-wrapper">
                    <img
                      src={image.src}
                      alt={`Loose layout item ${imgIndex}`}
                      style={{ width: `${imageWidth}px`, height: `${rowHeight}px` }}
                      loading="lazy" // 浏览器原生懒加载
                    />
                  </div>
                );
              })}
            </div>
          );
        })}
      </div>
    );
  };

  const renderGridLayout = () => (
    <div className="image-gallery grid">
      {images.map((image, index) => (
        <div key={index} className="image-item">
          <img src={image.src} alt={`Grid item ${index}`} />
        </div>
      ))}
    </div>
  );

  return (
    <div className="gallery-container" ref={galleryContainerRef}>
      <div className="controls">
        <button onClick={() => setLayout('grid')} className={layout === 'grid' ? 'active' : ''}>
          网格模式
        </button>
        <button onClick={() => setLayout('loose')} className={layout === 'loose' ? 'active' : ''}>
          宽松模式
        </button>
      </div>
      {layout === 'grid' ? renderGridLayout() : renderLooseLayout(containerWidth)}
    </div>
  );
};

export default ImageGallery;
