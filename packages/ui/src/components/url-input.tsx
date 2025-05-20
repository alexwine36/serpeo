"use client";
import { Button } from "@repo/ui/components/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@repo/ui/components/command";
import { Input } from "@repo/ui/components/input";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@repo/ui/components/popover";

import { Globe } from "lucide-react";
import type React from "react";
import { useCallback, useState } from "react";
import { z } from "zod";
import { cn } from "../lib/utils.js";
type UrlInputProps = {
  onSubmit?: (url: string) => void;
  placeholder?: string;
  className?: string;
  previousUrls?: string[];
};

const UrlSchema = z.string().url();

export const UrlInput = ({
  onSubmit,
  placeholder,
  className,
  previousUrls = [],
}: UrlInputProps) => {
  const [open, setOpen] = useState(false);
  const [inputValue, setInputValue] = useState("");
  const [commandInputValue, setCommandInputValue] = useState("");

  const validateUrl = (url: string) => {
    if (!url) {
      return;
    }

    // Normalize URL (add https:// if missing)
    let normalizedUrl = url;
    if (!url.startsWith("http://") && !url.startsWith("https://")) {
      normalizedUrl = `https://${url}`;
    }

    const result = UrlSchema.safeParse(normalizedUrl);

    if (!result.success) {
      return;
    }

    return normalizedUrl;
  };

  const handleSubmit = (e?: React.FormEvent) => {
    if (e) {
      e.preventDefault();
    }

    if (!inputValue.trim()) {
      return;
    }

    const normalizedUrl = validateUrl(inputValue);
    if (onSubmit && normalizedUrl) {
      onSubmit(normalizedUrl);
    }

    setInputValue("");
    setOpen(false);
  };

  const handleSelect = (rawUrl: string) => {
    const url = validateUrl(rawUrl);
    if (!url) {
      return;
    }
    setInputValue(url);
    setOpen(false);
  };

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  const AddGroup = useCallback(() => {
    if (!validateUrl(commandInputValue)) {
      return null;
    }
    return (
      <CommandGroup
        heading="Add URL"
        className={cn(!validateUrl(commandInputValue) && "hidden")}
      >
        <CommandItem
          value={commandInputValue}
          onSelect={() => handleSelect(commandInputValue)}
          className={cn("flex items-center")}
        >
          <Globe className="mr-2 h-4 w-4" />
          <span className="truncate">{commandInputValue}</span>
        </CommandItem>
      </CommandGroup>
    );
  }, [commandInputValue]);

  return (
    <form
      onSubmit={handleSubmit}
      className={cn("flex w-full items-center space-x-2", className)}
    >
      <div className="relative flex-1">
        <Popover open={open} modal onOpenChange={setOpen}>
          <PopoverTrigger asChild>
            <div className="relative w-full">
              <Globe className="-translate-y-1/2 absolute top-1/2 left-3 h-4 w-4 text-muted-foreground" />
              <Input
                value={inputValue}
                onChange={(e) => setInputValue(e.target.value)}
                placeholder={placeholder}
                className="w-full pr-10 pl-10"
                onFocus={() => setOpen(true)}
              />
              {inputValue && (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="absolute top-0 right-0 h-full px-3 py-2 hover:bg-transparent"
                  onClick={() => setInputValue("")}
                >
                  <span className="sr-only">Clear</span>
                  <span className="text-muted-foreground text-sm">×</span>
                </Button>
              )}
            </div>
          </PopoverTrigger>
          <PopoverContent
            className="w-[var(--radix-popover-trigger-width)] p-0"
            align="start"
          >
            <Command>
              <CommandInput
                onValueChange={(v) => {
                  setCommandInputValue(v);
                }}
                placeholder="Search URLs..."
              />
              <CommandList>
                <CommandEmpty>Invalid URL Address</CommandEmpty>
                <CommandGroup heading="Recent URLs">
                  {previousUrls.map((url, index) => (
                    <CommandItem
                      key={index}
                      value={url}
                      onSelect={() => handleSelect(url)}
                      className="flex items-center"
                    >
                      <Globe className="mr-2 h-4 w-4" />
                      <span className="truncate">{url}</span>
                    </CommandItem>
                  ))}
                </CommandGroup>
                <AddGroup />
              </CommandList>
            </Command>
          </PopoverContent>
        </Popover>
      </div>
      <Button type="submit">Go</Button>
    </form>
  );
};
