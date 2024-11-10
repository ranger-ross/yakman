import { t } from "../t";
import type { YakManLabelType } from "$lib/types/types";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";
import { z } from "zod";

const BASE_URL = getYakManBaseApiUrl();

export const CreateLabelSchema = z.object({
    name: z.string(),
    description: z.string(),
    options: z.array(z.string()),
});

export type CreateLabel = z.infer<typeof CreateLabelSchema>;


export const UpdateLabelSchema = z.object({
    name: z.string(),
    description: z.string(),
    options: z.array(z.string()),
});

export type UpdateLabel = z.infer<typeof UpdateLabelSchema>;


export const labels = t.router({
    fetchLabels: t.procedure
        .query(async ({ ctx }): Promise<YakManLabelType[]> => {
            const response = await fetch(`${BASE_URL}/v1/labels`, {
                headers: createYakManAuthHeaders(ctx.accessToken)
            });

            return await response.json();
        }),

    createLabel: t.procedure
        .input(CreateLabelSchema)
        .mutation(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/labels`, {
                method: 'PUT',
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(input)
            });
            if (response.status != 200) {
                throw new Error(await response.text())
            }
        }),
    updateLabel: t.procedure
        .input(z.object({
            id: z.string(),
            payload: UpdateLabelSchema
        }))
        .mutation(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/labels/${input.id}`, {
                method: 'POST',
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(input.payload)
            });
            if (response.status != 200) {
                throw new Error(await response.text())
            }
        }),
    deleteLabel: t.procedure
        .input(z.object({
            id: z.string(),
        }))
        .mutation(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/labels/${input.id}`, {
                method: 'DELETE',
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                },
            });
            if (response.status != 200) {
                throw new Error(await response.text())
            }
        })

});
