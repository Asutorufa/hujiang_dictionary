'use client';

import { useCallback, useEffect, useState } from "react";

export function useSpeechSynthesis() {
    const [voices, setVoices] = useState<SpeechSynthesisVoice[]>([]);
    const [speaking, setSpeaking] = useState(false);
    const [supported, setSupported] = useState(false);
    const [voice, saveVoice] = useState<SpeechSynthesisVoice | null>(null);

    useEffect(() => {
        if ("speechSynthesis" in window) {
            setSupported(true);

            const loadVoices = () => {
                const allVoices = window.speechSynthesis.getVoices();
                setVoices(allVoices);
            };

            loadVoices();
            window.speechSynthesis.onvoiceschanged = loadVoices;
        } else {
            setSupported(false);
        }
    }, []);

    const speak = useCallback((text: string, options = { rate: 1, pitch: 1, volume: 1 }) => {
        if (!supported) return;

        const { rate = 1, pitch = 1, volume = 1 } = options;

        const utterance = new SpeechSynthesisUtterance(text);
        if (voice) {
            utterance.voice = voice;
            console.log("use voice", voice.name, voice.voiceURI);
        }
        utterance.rate = rate;
        utterance.pitch = pitch;
        utterance.volume = volume;

        utterance.onstart = () => setSpeaking(true);
        utterance.onend = () => setSpeaking(false);

        window.speechSynthesis.speak(utterance);
    }, [supported, voice]);

    const cancel = useCallback(() => {
        if (supported) {
            window.speechSynthesis.cancel();
            setSpeaking(false);
        }
    }, [supported]);

    const pause = useCallback(() => {
        if (supported) {
            window.speechSynthesis.pause();
            setSpeaking(false);
        }
    }, [supported]);

    const resume = useCallback(() => {
        if (supported) {
            window.speechSynthesis.resume();
            setSpeaking(true);
        }
    }, [supported]);

    const setVoice = useCallback((voice: string) => {
        if (supported) {
            const v = voices.find((v) => v.voiceURI === voice || v.name === voice);
            if (v) {
                saveVoice(v);
                console.log("set voice", v.name, v.voiceURI);
            } else {
                console.log("voice not found", voice);
                saveVoice(null);
            }
        }
    }, [supported, voices, saveVoice]);

    return {
        supported,
        voices,
        speak,
        cancel,
        speaking,
        pause,
        resume,
        voice,
        setVoice,
    };
}