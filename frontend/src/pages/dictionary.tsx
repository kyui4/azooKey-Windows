import { useEffect, useState, type FormEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

type Entry = { reading: string; word: string };
type Update = { entries: Entry[]; applied: boolean };

export const Dictionary = () => {
    const [entries, setEntries] = useState<Entry[]>([]);
    const [reading, setReading] = useState("");
    const [word, setWord] = useState("");
    const [busy, setBusy] = useState(false);
    const [loaded, setLoaded] = useState(false);
    const [error, setError] = useState("");

    useEffect(() => {
        invoke<Entry[]>("get_dictionary").then(data => {
            setEntries(data);
            setLoaded(true);
        }).catch(error => setError(`辞書を読み込めませんでした: ${String(error)}`));
    }, []);

    const update = async (command: string, entry: Entry) => {
        setBusy(true);
        setError("");
        try {
            const result = await invoke<Update>(command, entry);
            setEntries(result.entries);
            toast(result.applied ? "辞書を更新しました" : "辞書を保存しました", {
                description: result.applied ? "次の変換から反映されます" : "変換エンジンへの反映に失敗しました。azooKeyを再起動してください。",
            });
            return true;
        } catch (error) {
            setError(String(error));
            return false;
        } finally {
            setBusy(false);
        }
    };

    const add = async (event: FormEvent) => {
        event.preventDefault();
        if (await update("add_dictionary_entry", { reading, word })) {
            setReading("");
            setWord("");
        }
    };

    return <section className="space-y-4">
        <h1 className="text-sm font-bold">ユーザー辞書</h1>
        <p className="text-sm text-muted-foreground">名前や専門用語を名詞として登録します。最大1000件です。</p>
        <form onSubmit={add} className="space-y-3 rounded-md border p-4">
            <label className="block text-sm" htmlFor="dictionary-reading">読み（ひらがな・カタカナ・アルファベット）</label>
            <Input id="dictionary-reading" value={reading} onChange={e => setReading(e.target.value)} placeholder="あずーきー" required disabled={busy || !loaded} />
            <label className="block text-sm" htmlFor="dictionary-word">単語</label>
            <Input id="dictionary-word" value={word} onChange={e => setWord(e.target.value)} placeholder="azooKey" required disabled={busy || !loaded} />
            <Button type="submit" disabled={busy || !loaded || !reading.trim() || !word.trim()}>登録</Button>
        </form>
        {error && <p role="alert" className="text-sm text-destructive">{error}</p>}
        {!loaded && !error && <p role="status">読み込み中…</p>}
        {loaded && entries.length === 0 && <p className="text-sm text-muted-foreground">登録された単語はありません。</p>}
        {entries.length > 0 && <div className="overflow-x-auto rounded-md border">
            <table className="w-full text-left text-sm">
                <caption className="sr-only">登録済みの単語</caption>
                <thead><tr className="border-b"><th scope="col" className="p-3">読み</th><th scope="col" className="p-3">単語</th><th scope="col" className="p-3">操作</th></tr></thead>
                <tbody>{entries.map(entry => <tr key={JSON.stringify(entry)} className="border-b last:border-0">
                    <td className="p-3 break-all">{entry.reading}</td><td className="p-3 break-all">{entry.word}</td>
                    <td className="p-3"><Button variant="secondary" disabled={busy} aria-label={`${entry.word}（${entry.reading}）を削除`} onClick={() => update("delete_dictionary_entry", entry)}>削除</Button></td>
                </tr>)}</tbody>
            </table>
        </div>}
    </section>;
};
