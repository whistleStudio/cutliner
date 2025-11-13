import { useState, useEffect } from 'react';
import { InputNumber, Radio, Checkbox, Button } from 'antd';
// import type { CheckboxGroupProps } from 'antd/es/checkbox';
import { invoke } from '@tauri-apps/api/core';
import { listen, emit } from '@tauri-apps/api/event';
import "./CfgMenu.scss"
import { open } from '@tauri-apps/plugin-shell';

const initialCfgs: Record<string, number> = {
  threshold: 128,
  bleed: 0,
  isDeleteInner: 0,
  smooth: 0,
  offset: 0,
  simplify: 0,
  fillHoles: 0,  
}

export default function CfgMenu() {
  const [selectedMode, setSelectedMode] = useState<string>('bg_remove');
  const [cfgs, setCfgs] = useState<Record<string, number>>({
    ...initialCfgs
  });
  const [isShowCut, setIsShowCut] = useState<boolean>(false);
  const mySiteHref = "https://space.bilibili.com/619221106?spm_id_from=333.1007.0.0";

  useEffect(() => {
    let unlisten: () => void;
    const thresholdListener =  async () => {
      unlisten = await listen<number>('set-otsu-threshold', event => {
        console.log("Received Otsu threshold:", event.payload);
        setCfgs({
          ...initialCfgs,
          threshold: event.payload || 128
        })
      });
    };
    thresholdListener();
    return () => {
      if (unlisten) { unlisten(); }
    };
  }, []);

  const modeOpts = [
    { label: '去背', value: 'bg_remove' },
    { label: '外轮廓', value: 'contour_outer' },
    { label: '全部轮廓', value: 'contour_all' },
  ];

  const handleCfgChange = (key: string, value: number) => {
    setCfgs(preCfgs => ({
      ...preCfgs,
      [key]: value,
    }));
  }

  const handlePreview = async () => {
    // console.log("Previewing with settings:", cfgs);
    try {
      const res = await invoke<string>('solve', {mode: selectedMode, cfgs});
      emit('img-changed', res);
    } catch (err) {
      console.error("Error during preview:", err);
      alert("预览失败");
    }
  }

  const handleLogoClick = () => {
    console.log("Logo clicked, opening website...");
    open(mySiteHref);
  }; 

  return (
    <ul style={{width: "30%", height: "100%", display: "flex", flexDirection: "column", justifyContent: "center", alignItems: "flex-end", padding: "20px 20px 0 20px", boxSizing: "border-box"}}>
      <li style={{width: "100%"}}>
        <Radio.Group 
        block options={modeOpts} optionType='button' buttonStyle='solid' size='large' style={{width: "100%"}}
        value={selectedMode} onChange={(e) => setSelectedMode(e.target.value)}
        />
      </li>
      <li>
        <InputNumber min={0} max={255} value={Math.trunc(cfgs.threshold)} onChange={val => {handleCfgChange("threshold", val??0)}}/>
        <span>二值化阈值</span>
      </li>
      {
        selectedMode === 'bg_remove' && (
          <>
            <li>
              <InputNumber min={0}  max={100} value={Math.trunc(cfgs.bleed)} onChange={val => {handleCfgChange("bleed", val??0)}}/>
              <span>出血</span>
            </li>
            <li>
              <InputNumber min={0} max={100} value={Math.trunc(cfgs.smooth)} onChange={val => {handleCfgChange("smooth", val??0)}}/>
              <span>平滑</span>
            </li>
            <li>
              <InputNumber<number> defaultValue={0} min={0} max={1000} formatter={val => `${val}‰`} parser={val => Number(val?.replace("‰", "").trim() || 0)} onChange={val => {handleCfgChange("simplify", val??0)}}/>
              <span>简化</span>
            </li>
            <li>
              <Checkbox style={{fontSize: "22px", display: "flex", alignItems: "center"}} 
              checked={cfgs.isDeleteInner === 1} onChange={e => {handleCfgChange("isDeleteInner", e.target.checked ? 1 : 0)}}>去除内部</Checkbox>
            </li>
          </>
        )
      }
      {
        selectedMode !== 'bg_remove' && (
          <>
            <li>
              <InputNumber min={0} max={255} value={Math.trunc(cfgs.smooth)} onChange={val => {handleCfgChange("smooth", val??0)}}/>
              <span>平滑</span>
            </li>
            <li>
              <InputNumber min={0} max={100} value={Math.trunc(cfgs.offset)} onChange={val => {handleCfgChange("offset", val??0)}}/>
              <span>偏移</span>
            </li>
            <li>
              <InputNumber<number> defaultValue={0} min={0} max={1000} formatter={val => `${val}‰`} parser={val => Number(val?.replace("‰", "").trim() || 0)} onChange={val => {handleCfgChange("simplify", val??0)}}/>
              <span>简化</span>
            </li>
          </>
        )
      }
      {
        selectedMode === 'contour_all' && (
          <li>
            <InputNumber min={0}  max={100} value={Math.trunc(cfgs.fillHoles)} onChange={val => {handleCfgChange("fillHoles", val??0)}}/>
            <span>孔洞填充</span>
          </li>
        )
      }
      <li style={{width: "100%", marginTop: "auto", marginBottom: "7px", flexDirection: "column", alignItems: "flex-end", position: "relative"}}>
        {isShowCut && <img src="/cut.png" alt="cut cut~" className='cut' height={60} style={{position: "absolute", left: 10, top: 42}}/>}
        <img className="logo" src="/icon.png" alt="Mr. Hungry" height={150} onMouseEnter={()=>{setIsShowCut(true);}} onMouseLeave={() => {setIsShowCut(false);}} onClick={handleLogoClick}/>
        <Button color='cyan' variant="solid" size='large' style={{width: "100%", marginTop: "50px"}} onClick={handlePreview}>预览</Button>
      </li>
    </ul>
  );
}