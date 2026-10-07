import { Textarea } from "@/components/ui/textarea";
import { Switch } from "@/components/ui/switch";
import { Bot, User, Cpu } from "lucide-react";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select"
import { useEffect, useState } from "react";
import { toast } from "sonner"
import { invoke } from '@tauri-apps/api/core';
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from "@/components/ui/tooltip"

const ToolTipSelectItem = ({
    name,
    value,
    disabled,
    tooltip
}: {
    name: string;
    value: string;
    disabled: boolean;
    tooltip: string;
}) => {
    return (
        <TooltipProvider>
            <Tooltip>
                <TooltipTrigger>
                    <SelectItem value={value} disabled={disabled}>
                        {name}
                    </SelectItem>
                </TooltipTrigger>
                {disabled && <TooltipContent side="left">
                    {tooltip}
                </TooltipContent>}
            </Tooltip>
        </TooltipProvider>
    )
}

export const Zenzai = () => {
    const [value, setValue] = useState({
        enable: false,
        profile: "",
        backend: "",
        gpu_layers: 99,
    });

    const [capability, setCapability] = useState({
        cpu: true,
        cuda: false,
        vulkan: false,
    });

    // Load config on component mount
    useEffect(() => {
        invoke<any>("get_config")
            .then((data) => {
                const zenzai = data.zenzai;
                setValue({
                    enable: zenzai.enable,
                    profile: zenzai.profile,
                    backend: zenzai.backend,
                    gpu_layers: zenzai.gpu_layers ?? 99,
                });
            })
            .catch(() => {
                // Keep default values if config fetch fails
            });

        invoke("check_capability").then((capability: any) => {
            setCapability({
                cpu: capability["cpu"],
                cuda: capability["cuda"],
                vulkan: capability["vulkan"],
            });
        })
    }, []);

    const updateConfig = async (updater: (config: any) => void) => {
        try {
            const data = await invoke<any>("get_config");
            updater(data);
            await invoke("update_config", { newConfig: data });
            return data;
        } catch (error) {
            toast("設定の更新に失敗しました", { description: String(error) });
            return null;
        }
    };

    const handleZenzaiChange = async () => {
        const data = await updateConfig((data) => {
            data.zenzai.enable = !value.enable;
        });
        
        if (data) {
            setValue((prev) => ({ ...prev, enable: data.zenzai.enable }));
        }
    };

    const handleProfileChange = (event: React.ChangeEvent<HTMLTextAreaElement>) => {
        const newProfile = event.target.value;
        setValue((prev) => ({ ...prev, profile: newProfile }));
        
        updateConfig((data) => {
            data.zenzai.profile = newProfile;
        });
    };

    const handleBackendChange = async (backend: string) => {
        const data = await updateConfig((data) => {
            data.zenzai.backend = backend;
        });
        
        if (data) {
            setValue((prev) => ({ ...prev, backend }));
            toast("バックエンドが変更されました", {
                description: "変更を適用するには、PCを再起動してください",
                duration: 10000,
            });
        }
    };

    const handleGpuLayersChange = async (layers: string) => {
        const gpu_layers = Number(layers);
        const data = await updateConfig(data => { data.zenzai.gpu_layers = gpu_layers; });
        if (data) {
            setValue(prev => ({ ...prev, gpu_layers }));
            toast("GPUへの割り当てを変更しました", { description: "変更を適用するには、PCを再起動してください" });
        }
    };

    return (
        <div className="space-y-8">
            <section className="space-y-2">
                <h1 className="text-sm font-bold text-foreground">Zenzai</h1>
                <div className="flex items-center space-x-4 rounded-md border p-4">
                    <Bot />
                    <div className="flex-1 space-y-1">
                        <p className="text-sm font-medium leading-none">
                            Zenzaiを有効化
                        </p>
                        <p className="text-xs text-muted-foreground">
                            Zenzaiを有効にして、変換精度を向上させます
                        </p>
                    </div>
                    <Switch checked={value.enable} onCheckedChange={handleZenzaiChange} />
                </div>
                <div className="space-y-4 rounded-md border p-4">
                    <div className="flex items-center space-x-4 ">
                        <User />
                        <div className="flex-1 space-y-1">
                            <p className="text-sm font-medium leading-none">
                                変換プロファイル
                            </p>
                            <p className="text-xs text-muted-foreground">
                                Zenzaiで利用されるユーザー情報を設定します
                            </p>
                        </div>
                    </div>
                    <Textarea placeholder="例）山田太郎、数学科の学生。" value={value.profile} disabled={!value.enable} onChange={handleProfileChange} />
                </div>
                <div className="flex items-center space-x-4 rounded-md border p-4">
                    <Cpu />
                    <div className="flex-1 space-y-1">
                        <p className="text-sm font-medium leading-none">
                            バックエンド
                        </p>
                        <p className="text-xs text-muted-foreground">
                            Zenzaiを利用するバックエンドを選択します
                        </p>
                    </div>
                    <Select disabled={!value.enable} value={value.backend} onValueChange={handleBackendChange}>
                        <SelectTrigger className="w-48">
                            <SelectValue placeholder="バックエンドを選択" />
                        </SelectTrigger>
                        <SelectContent className="flex flex-col">
                            <ToolTipSelectItem name="CPU (非推奨)" value="cpu" disabled={!capability.cpu} tooltip="" />
                            <ToolTipSelectItem name="CUDA (NVIDIA GPU)" value="cuda" disabled={!capability.cuda} tooltip="CUDA Toolkit 12をインストールする必要があります" />
                            <ToolTipSelectItem name="Vulkan" value="vulkan" disabled={!capability.vulkan} tooltip="お使いのPCはVulkanに対応していません" />
                        </SelectContent>
                    </Select>
                </div>
            </section>
            <section className="flex items-center space-x-4 rounded-md border p-4">
                <div className="flex-1 space-y-1">
                    <h2 className="text-sm font-medium">GPUに載せる層数</h2>
                    <p className="text-xs text-muted-foreground">通常は全層を選択します。GPUメモリが足りない場合や動作が遅い場合は減らしてください。速度はGPUにより異なります。</p>
                </div>
                <Select disabled={!value.enable || value.backend === "cpu"} value={String(value.gpu_layers)} onValueChange={handleGpuLayersChange}>
                    <SelectTrigger className="w-48" aria-label="GPUに載せる層数"><SelectValue /></SelectTrigger>
                    <SelectContent>
                        <SelectItem value="99">全層（通常設定）</SelectItem>
                        {[0, 8, 16, 24, 32].map(layers => <SelectItem key={layers} value={String(layers)}>{layers}層</SelectItem>)}
                    </SelectContent>
                </Select>
            </section>
        </div>
    )
}
