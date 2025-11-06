import {useState, useEffect, use} from "react";
import {Image, Button} from "antd"
import { UploadOutlined } from '@ant-design/icons';
// import type { UploadProps, UploadFile } from 'antd';
import { invoke } from "@tauri-apps/api/core";
import { listen, emit } from "@tauri-apps/api/event";
import "./SelectWin.scss"

export default function SelectWin() {
  const [SelectImgSrc, setSelectImgSrc] = useState<string | null>(null);

  useEffect(() => {
    let unlisten: () => void;
    const dataListener =  async () => {
      unlisten = await listen<Uint8Array>('imgData-changed', event => {
        console.log("Received image data event:", event);
        setImg(event.payload);
      });
    };
    dataListener();
    return () => {
      if (unlisten) { unlisten(); }
    };
  }, []);
  
  /* 选择本地图片 */
  const handleSelectImage = async () => {
    try {
      const [imageData, otsuThreshold] = await invoke<[Uint8Array|null, number]>('select_image');
      console.log("Selected file path from Rust:", imageData);
      if (imageData) {
        setImg(imageData);
        emit('set-otsu-threshold', otsuThreshold);
      }
    } catch (err) { console.error("Error selecting image:", err); }
  };

  // const beforeUpload = (file: UploadFile) => {
  //   setSelectImage(file);
  //   console.log("selected file:", file);
  //   return false;
  // };
  function setImg(data: Uint8Array) {
    const blob = new Blob([new Uint8Array(data)], { type: 'image/png' });
    setSelectImgSrc(URL.createObjectURL(blob));
  }

  return (
    <div className="flex-col-align-center" style={{width: '70%', height: '100%', marginRight: "5px", borderRight: "1px solid #eee"}}>
      <Image height={720} src={SelectImgSrc || "/demo.jpg"} className="preview-container"/>
      <Button icon={<UploadOutlined />} onClick={handleSelectImage}
      style={{marginTop: "20px", fontSize: "20px", lineHeight: "40px"}}
      >Select Image</Button>
      {/* <Upload maxCount={1} beforeUpload={beforeUpload} showUploadList={false}>
        <Button icon={<UploadOutlined />}>Select Image</Button>
      </Upload> */}
    </div>
  );
}