CREATE OR REPLACE VIEW "DBAMV"."ALLERGY"
(
    "allergy_encounter_code",
    "allergy_patient_code",
    "allergy_substance_name",
    "allergy_status",
    "allergy_date",
    "allergy_time",
    "allergy_provider_code",
    "allergy_manifestation",
    "allergy_severity",
    "allergy_updated_date",
    "allergy_responsible",     
    "allergy_reason"
) AS
SELECT 
    NULL AS "allergy_encounter_code",
    HSP.CD_PACIENTE AS "allergy_patient_code",                                                             
    SUB.DS_SUBSTANCIA AS "allergy_substance_name",  
    HSP.SN_ATIVO AS "allergy_status",                                                               
    TO_CHAR(HSP.DH_CRIACAO, 'YYYY-MM-DD') AS "allergy_date",                                               
    TO_CHAR(HSP.DH_CRIACAO, 'HH24:MI:SS') AS "allergy_time",                                               
    U.CD_PRESTADOR AS "allergy_provider_code",                                                             
    CASE HSP.TP_ALERGIA                                                                                    
        WHEN 'M' THEN 'MEDICAMENTO'                                                                        
        WHEN 'A' THEN 'ALIMENTAR'                                                                          
        ELSE 'OUTROS'                                                                                      
    END AS "allergy_manifestation",                                                                        
    CASE HSP.TP_SEVERIDADE                                                                                 
        WHEN 'L' THEN 'LEVE'                                                                               
        WHEN 'M' THEN 'MODERADO'                                                                           
        WHEN 'G' THEN 'GRAVE'                                                                              
        ELSE TO_CHAR(HSP.TP_SEVERIDADE)                                                                         
    END AS "allergy_severity",                                                                                                                                         
    CANC.DH_CANCELAMENTO AS "allergy_updated_date",
    CANC.CD_USUARIO_CANCELAMENTO AS "allergy_responsible",     
    CANC.DS_JUSTIFICATIVA_CANC  AS "allergy_reason"                                                                                                                                                                    
FROM DBAMV.HIST_SUBS_PAC HSP
LEFT JOIN DBAMV.SUBSTANCIA SUB ON HSP.CD_SUBSTANCIA = SUB.CD_SUBSTANCIA
LEFT JOIN DBASGU.USUARIOS U ON U.CD_USUARIO = HSP.CD_USUARIO_CRIACAO
LEFT JOIN DBAMV.HIST_SUBS_PAC CANC ON CANC.CD_HIST_SUBS_PAC_CANC = HSP.CD_HIST_SUBS_PAC
WHERE 
HSP.CD_HIST_SUBS_PAC_CANC IS NULL AND HSP.CD_USUARIO_CANCELAMENTO IS NULL AND HSP.DH_CANCELAMENTO IS NULL
AND HSP.DH_CRIACAO IS NOT NULL
ORDER BY HSP.DH_CRIACAO DESC