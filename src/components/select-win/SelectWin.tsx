import {useState, useEffect} from "react";
import {Image, Button} from "antd"
import { DownloadOutlined, UploadOutlined } from '@ant-design/icons';
// import type { UploadProps, UploadFile } from 'antd';
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { listen, emit } from "@tauri-apps/api/event";
import "./SelectWin.scss"


export default function SelectWin() {
  const [SelectImgSrc, setSelectImgSrc] = useState<string | null>(null);

  useEffect(() => {
    let unlisten: () => void;
    const dataListener =  async () => {
      unlisten = await listen<string>('img-changed', event => {
        // setImg(event.payload);
        console.log("Converted image source:", event.payload, convertFileSrc(event.payload));
        setSelectImgSrc(convertFileSrc(event.payload));
      });
    };
    dataListener();
    invoke('init', { imgNameWithExt: "input.jpg" }); // 初始化应用内包含的测试默认图片
    return () => {
      if (unlisten) { unlisten(); }
    };
  }, []);
  
  /* 选择本地图片 */
  const handleSelectImage = async () => {
    try {
      const [imageSrc, otsuThreshold] = await invoke<[string|null, number]>('select_image');
      console.log("Selected file path from Rust:", imageSrc);
      if (imageSrc) {
        // setImg(imageData);
        setSelectImgSrc(convertFileSrc(imageSrc));
        emit('set-otsu-threshold', otsuThreshold);
      }
    } catch (err) { console.error("Error selecting image:", err); }
  };

  /* 保存图片 */
  const handleSaveImage = async () => {
    try {
      await invoke('save_image');
    } catch (err) { console.error("Error saving image:", err); }
  };

  return (
    <div className="flex-col-align-center" style={{width: '70%', height: '100%', marginRight: "5px", borderRight: "1px solid #eee"}}>
      <Image height={720} src={SelectImgSrc || convertFileSrc("")} className="preview-container"/>
      <div className="flex-row-align-center" style={{width: "60%", justifyContent: "space-evenly"}}>
        <Button icon={<UploadOutlined />} onClick={handleSelectImage}
        style={{marginTop: "20px", fontSize: "20px", lineHeight: "40px", width: "30%"}}
        >Upload</Button>
        <Button icon={<DownloadOutlined />} onClick={handleSaveImage}
        style={{marginTop: "20px", fontSize: "20px", lineHeight: "40px", width: "30%"}}
        >Save</Button>
      </div>


      {/* <Upload maxCount={1} beforeUpload={beforeUpload} showUploadList={false}>
        <Button icon={<UploadOutlined />}>Select Image</Button>
      </Upload> */}
    </div>
  );
}