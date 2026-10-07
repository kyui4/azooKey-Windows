import { Button } from "@/components/ui/button";
import { RefreshCcw, ExternalLink } from "lucide-react";

export const General = () => {
    return (
        <div className="space-y-8">
            <section className="space-y-2 rounded-md border p-4">
                <h1 className="text-sm font-bold">日本語・英数の切り替え</h1>
                <p className="text-sm">無変換キーで英数、変換キーで日本語に切り替えます。</p>
                <p className="text-xs text-muted-foreground">入力途中で英数へ切り替えると、現在の変換候補を確定します。同じモードのキーを押しても切り替わりません。</p>
            </section>
            <section className="space-y-2">
                <h1 className="text-sm font-bold text-foreground">バージョンと更新プログラム</h1>
                <div className="flex items-center space-x-4 rounded-md border p-4">
                    <RefreshCcw />
                    <div className="flex-1 space-y-1">
                        <p className="text-sm font-medium leading-none">
                            v0.1.0-alpha.1
                        </p>
                    </div>
                    <Button  variant="secondary">
                        <a href="https://github.com/fkunn1326/azooKey-Windows/releases" className="flex items-center gap-x-2" target="_blank" rel="noopener noreferrer">
                            <ExternalLink />
                            更新を確認する
                        </a>
                    </Button>
                </div>
            </section>
            {/* <section className="space-y-2">
                <h1 className="text-sm font-bold text-foreground">診断とフィードバック</h1>
                <div className="flex items-center space-x-4 rounded-md border p-4">
                    <FileChartColumn />
                    <div className="flex-1 space-y-1">
                        <p className="text-sm font-medium leading-none">
                            診断データ
                        </p>
                        <p className="text-xs text-muted-foreground">
                            診断データを保存し、バグの修正に役立てます
                        </p>
                    </div>
                    <Switch />
                </div>
            </section> */}
        </div>
    )
}
