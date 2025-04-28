"use client";

import {
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@repo/ui/components/form";
import { Input } from "@repo/ui/components/input";
// import { Label } from "../components/label";
import { cn } from "@repo/ui/lib/utils";
import type * as React from "react";
import {
  type Control,
  type FieldValues,
  type Path,
  useFormContext,
} from "react-hook-form";

type BaseFormInputProps<T extends FieldValues> = {
  label: string;
  name: Path<T>;
  description?: string;
  control?: Control<T>;
  className?: string;
};

type TextInputProps = {
  type?: "text" | "email" | "password" | "url" | "tel";
  placeholder?: string;
};

type NumberInputProps = {
  type: "number";
  min?: number;
  max?: number;
  step?: number;
};

type DateInputProps = {
  type: "date" | "datetime-local" | "time";
  min?: string;
  max?: string;
};

type FormInputProps<T extends FieldValues> = BaseFormInputProps<T> &
  Omit<React.InputHTMLAttributes<HTMLInputElement>, "name" | "type"> &
  (TextInputProps | NumberInputProps | DateInputProps);

export function FormInput<T extends FieldValues>({
  label,
  name,
  description,
  className,
  control: externalControl,
  type = "text",
  ...props
}: FormInputProps<T>) {
  const formContext = useFormContext<T>();
  const control = externalControl || formContext?.control;

  if (!control) {
    throw new Error(
      "FormInput must be used within a Form or have a control prop provided"
    );
  }

  return (
    <FormField
      control={control}
      name={name}
      render={({ field }) => (
        <FormItem className="space-y-2">
          <FormLabel>{label}</FormLabel>
          <FormControl>
            <Input
              type={type}
              {...field}
              {...props}
              id={name}
              className={cn(className)}
              aria-describedby={description ? `${name}-description` : undefined}
            />
          </FormControl>
          {description && <FormDescription>{description}</FormDescription>}
          <FormMessage />
        </FormItem>
      )}
    />
  );
}
