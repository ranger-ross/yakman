import { z } from "zod";
import { t } from "../t";

export const createProject = t.procedure
    .input(z.string())
    .mutation(async ({ input }) => {
        console.log(input);

        // TODO: Save Project with API

    })