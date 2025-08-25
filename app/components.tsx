import { addToast, Button, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader, Textarea, useDisclosure } from "@heroui/react";
import { FC, useEffect, useState } from "react";

export const SaveWordModal: FC<{
    open: boolean,
    onChange: (open: boolean) => void,
    word?: string,
    explain?: string,
}> = ({ open, onChange, word, explain }) => {
    const [saving, setSaving] = useState(false)
    const { isOpen, onOpen, onOpenChange } = useDisclosure();

    useEffect(() => {
        if (open) {
            onOpen()
        }
    }, [open, onOpen])

    const [newWord, setNewWord] = useState(word || "");
    const [newExplain, setNewExplain] = useState(explain || "");

    useEffect(() => {
        setNewWord(word || "");
        setNewExplain(explain || "");
    }, [word, explain])

    return <Modal isOpen={isOpen} backdrop="blur" placement="top-center" onOpenChange={(p) => {
        onChange(p)
        onOpenChange()
    }}>
        <ModalContent>
            {(onClose) => (
                <>
                    <ModalHeader className="flex flex-col gap-1">Save Word</ModalHeader>
                    <ModalBody>
                        <Textarea
                            label="Word"
                            isInvalid={newWord.length === 0}
                            errorMessage={"Word is empty"}
                            value={newWord}
                            onChange={(p) => setNewWord(p.target.value)}
                            placeholder="Enter Word"
                            variant="bordered"
                        />
                        <Textarea
                            label="Explain"
                            isInvalid={newExplain.length === 0}
                            errorMessage={"Explain is empty"}
                            value={newExplain}
                            onChange={(p) => setNewExplain(p.target.value)}
                            placeholder="Enter explain"
                            variant="bordered"
                        />
                    </ModalBody>
                    <ModalFooter>
                        <Button color="danger" variant="flat" onPress={onClose}>
                            Close
                        </Button>
                        <Button color="primary" isLoading={saving} onPress={() => {
                            if (newWord.length === 0 || newExplain.length === 0) return
                            setSaving(true)
                            saveWord(newWord, newExplain, () => {
                                setSaving(false)
                                onClose()
                            })
                        }}>
                            Save
                        </Button>
                    </ModalFooter>
                </>
            )}
        </ModalContent>
    </Modal>
}

export type ListWordResponse = {
    word: string,
    explain: string,
    add_time: number,
    update_time: number,
    reminder_time: number,
    anki_count: number,
    priority: number
}

export async function wordRequest<T>(path: string, body: string, callback: (data?: T, error?: string) => void) {
    fetch(path, {
        method: "POST",
        headers: {},
        body: body,
    })
        .then((res) => res.json() as Promise<T>)
        .then((data) => {
            callback(data, undefined);
        })
        .catch((error) => {
            callback(undefined, error.message);
            addToast({
                title: `Words Request Error(${path})`,
                description: error.message,
                color: "danger",
                timeout: 0
            });
        });
}

export async function queryWord(page: number, size: number, order_by: string, callback: (data?: ListWordResponse[], error?: string) => void) {
    await wordRequest<ListWordResponse[]>("/word/list", JSON.stringify({
        page_size: size,
        page_number: page,
        order_by: order_by
    }), callback)
}

type CountWordResponse = {
    size: number,
}

export async function countWord(callback: (size?: number, error?: string) => void) {
    await wordRequest<CountWordResponse>("/word/count", "", (data, error) => {
        callback(data?.size, error);
    })
}

export async function saveWord(word: string, explain: string, callback: (error?: string) => void) {
    await wordRequest<object>("/word/save", JSON.stringify({
        word: word,
        explain: explain
    }), (_, error) => {
        callback(error);
    });
}

export async function incrementRemindCount(word: string, callback: (error?: string) => void) {
    await wordRequest<object>("/word/remind_count_increment", JSON.stringify({
        word: word
    }), (_, error) => {
        callback(error);
    });
}

export async function changePriority(word: string, priority: number, callback: (error?: string) => void) {
    await wordRequest<object>("/word/priority", JSON.stringify({
        word: word,
        priority: priority
    }), (_, error) => {
        callback(error);
    });
}
