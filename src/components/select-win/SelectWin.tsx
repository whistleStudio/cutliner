import {useState, useEffect} from "react";
import {Image, Button} from "antd"
import { DownloadOutlined, UploadOutlined, ReloadOutlined } from '@ant-design/icons';
// import type { UploadProps, UploadFile } from 'antd';
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { listen, emit } from "@tauri-apps/api/event";
import "./SelectWin.scss"

interface SelectWinProps {
  btnState: boolean;
  onChangeBtnState: (e: boolean) => void;
}

export default function SelectWin(props: SelectWinProps) {
  // const [selectImgSrc, setSelectImgSrc] = useState<string | null>(null);
  const [originalImgSrc, setOriginalImgSrc] = useState<string | null>(null);
  const [tempImgSrc, setTempImgSrc] = useState<string | null>(null);
  const [isReloaded, setIsReloaded] = useState<boolean>(false);

  useEffect(() => {
    let unlistenArr: Array<() => void> = [];
    const dataListener =  async () => {
      unlistenArr.push(await listen<string>('img-changed', event => {
        // setImg(event.payload);
        const imgSrc = convertFileSrc(event.payload);
        // setSelectImgSrc(imgSrc);
        setOriginalImgSrc(prev => prev || imgSrc); // 仅在初始为空时执行; 用回调避免闭包陷阱
        setTempImgSrc(imgSrc);
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
      props.onChangeBtnState(true);
      const [imageSrc, otsuThreshold] = await invoke<[string|null, number]>('select_image');
      console.log("Selected file path from Rust:", imageSrc);
      if (imageSrc) {
        // setImg(imageData);
        setTempImgSrc(convertFileSrc(imageSrc));
        setOriginalImgSrc(convertFileSrc(imageSrc));
        emit('set-otsu-threshold', otsuThreshold);
      }
    } catch (err) { console.error("Error selecting image:", err); alert("图片上传失败"); }
    props.onChangeBtnState(false);
  };

  /* 保存图片 */
  const handleSaveImage = async () => {
    props.onChangeBtnState(true);
    try {
      await invoke('save_image');
    } catch (err) { console.error("Error saving image:", err); alert("图片保存失败"); }
    props.onChangeBtnState(false);
  };

  /* 控制原图像显示，做对照用 */
  const handleReloadState = (sta: boolean) => !props.btnState && setIsReloaded(sta);

  return (
    <div className="flex-col-align-center" style={{width: '70%', height: '100%', marginRight: "5px", borderRight: "1px solid #eee"}}>
      <Image height={720} style={{objectFit: "contain"}} src={(isReloaded ? originalImgSrc : tempImgSrc) || ""} className="preview-container"/>
      <div className="flex-row-align-center" style={{width: "60%", justifyContent: "space-evenly", marginTop: "20px"}}>
        <Button icon={<UploadOutlined />} onClick={handleSelectImage} className="btn"
        style={{fontSize: "20px", lineHeight: "40px", width: "30%"}} disabled={props.btnState}
        >上传</Button>
        <ReloadOutlined className={`reload ${isReloaded?"mouse-down":""} ${props.btnState?"disabled":""}`} 
        onMouseDown={() => handleReloadState(true)} onMouseLeave={() => handleReloadState(false)} onMouseUp={() => handleReloadState(false)}/>
        <Button icon={<DownloadOutlined />} onClick={handleSaveImage} className="btn" disabled={props.btnState}
        style={{fontSize: "20px", lineHeight: "40px", width: "30%"}}
        >保存</Button>
      </div>
      {/* <Upload maxCount={1} beforeUpload={beforeUpload} showUploadList={false}>
        <Button icon={<UploadOutlined />}>Select Image</Button>
      </Upload> */}
    </div>
  );
}