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
    let unlistenArr: Array<() => void> = [];
    const dataListener =  async () => {
      unlistenArr.push(await listen<string>('img-changed', event => {
        // setImg(event.payload);
        console.log("Converted image source:", event.payload, convertFileSrc(event.payload));
        setSelectImgSrc(convertFileSrc(event.payload));
      }));
    };
    dataListener();
    invoke('init', { imgNameWithExt: "demo.jpg" }); // ⭐初始化应用内包含的测试默认图片
    // 微调：修改图片preview为中文
    const previewDiv = document.querySelector(".ant-image-mask-info");
    const previewTextEl = previewDiv?.childNodes[previewDiv?.childNodes.length - 1] as HTMLElement | undefined;
    if (previewTextEl) {
      previewTextEl.textContent = "预览";
    }

    return () => {
      if (unlistenArr.length > 0) { unlistenArr.forEach(unlisten => unlisten()); }
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
    } catch (err) { console.error("Error selecting image:", err); alert("图片上传失败"); }
  };

  /* 保存图片 */
  const handleSaveImage = async () => {
    try {
      await invoke('save_image');
    } catch (err) { console.error("Error saving image:", err); alert("图片保存失败"); }
  };

  return (
    <div className="flex-col-align-center" style={{width: '70%', height: '100%', marginRight: "5px", borderRight: "1px solid #eee"}}>
      <Image height={720} style={{objectFit: "contain"}} src={SelectImgSrc || convertFileSrc("")} className="preview-container"/>
      <div className="flex-row-align-center" style={{width: "60%", justifyContent: "space-evenly"}}>
        <Button icon={<UploadOutlined />} onClick={handleSelectImage} className="btn"
        style={{marginTop: "20px", fontSize: "20px", lineHeight: "40px", width: "30%"}}
        >上传</Button>
        <Button icon={<DownloadOutlined />} onClick={handleSaveImage} className="btn"
        style={{marginTop: "20px", fontSize: "20px", lineHeight: "40px", width: "30%"}}
        >保存</Button>
      </div>


      {/* <Upload maxCount={1} beforeUpload={beforeUpload} showUploadList={false}>
        <Button icon={<UploadOutlined />}>Select Image</Button>
      </Upload> */}
    </div>
  );
}