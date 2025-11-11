import { JSX, useEffect, useState } from "react";
import {  List, RowComponentProps } from 'react-window';
import "./App.css";
// import ImageGallery from "./ImageGallery";
import { invoke } from "@tauri-apps/api/core";

function App() {
  return (
    <main className="container">
      <h1>图片展示</h1>
      {/* <ImageGallery /> */}
      <TestImageFromRust />
    </main>
  );
}

function TestImageFromRust(): JSX.Element {

  const [imagePaths, setImagePaths] = useState<string[]>([]);
  // const [dimensions, setDimensions] = useState({ 
  //   height: window.innerHeight, 
  //   width: window.innerWidth 
  // });

  // 1. 在组件加载时，从 Rust 获取图片路径列表
  useEffect(() => {
    invoke<string[]>('get_image_paths')
      .then((paths) => {
        console.log(`从 Rust 获得了 ${paths.length} 张图片`);
        setImagePaths(paths);
      })
      .catch(console.error);
  }, []);

  const Row = ({ index, imagePaths, style }: RowComponentProps<{imagePaths: string[]}>) => {
    const path = imagePaths[index];
    const imageUrl = `http://asset.localhost/${path}`;

    // 'style' 是 react-window 传递的，用于正确定位
    return (
      <div style={style as React.CSSProperties} className="list-item">
        <img
          src={imageUrl}
          alt="local-thumbnail"
          loading="lazy" // 浏览器原生懒加载
          style={{ maxHeight: '120px' }}
        />
      </div>
    );
  };

  return (

      <List
        rowProps={{imagePaths}}
        rowHeight={130}
        rowCount={imagePaths.length}
        rowComponent={Row}
      />

  );
}

export default App;

